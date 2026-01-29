use std::{sync::mpsc::{self, Sender}, thread};
use pitch_detection::detector::{PitchDetector, mcleod::McLeodDetector, yin::YINDetector};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug)]
pub struct NoteDetector{
    pub note: String,
    pub frequency: f32,
    pub offset: f32,
}

impl NoteDetector{

    pub fn connect(sender: Sender<NoteDetector>) -> cpal::Stream {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No se encontró micrófono");
        let config = device.default_input_config().unwrap();

        let sample_rate = config.sample_rate() as usize;
        let (tx_audio, rx_audio) = mpsc::channel::<Vec<f32>>();
        
        thread::spawn(move || {
            const WINDOW_SIZE: usize = 1024;
            const PADDING: usize = WINDOW_SIZE / 2;
            const POWER_THRESHOLD: f32 = 0.15;
            const CLARITY_THRESHOLD: f32 = 0.6;

            let mut detector = McLeodDetector::new(WINDOW_SIZE, PADDING);
            let mut buffer: Vec<f32> = Vec::with_capacity(WINDOW_SIZE);

            while let Ok(chunk) = rx_audio.recv() {
                for sample in chunk {
                    buffer.push(sample as f32);
                }
                if buffer.len() >= WINDOW_SIZE {
                    let result = detector.get_pitch(
                        &buffer,
                        sample_rate,
                        POWER_THRESHOLD,
                        CLARITY_THRESHOLD
                    );
                    if let Some(pitch) = result {
                        let frequency = pitch.frequency as f32;
                        let (key, octave) = Self::calculate_key_and_octave(frequency);
                        let result = NoteDetector {
                            frequency,
                            note: Self::format_note(key, octave),
                            offset: Self::cents_offset_by_numerical_key(key, frequency)
                        };
                        if sender.send(result).is_err() {
                            break;
                        }
                    }
                    buffer.clear();
                }
            }
        });

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let data_vec = data.to_vec();
                let _ = tx_audio.send(data_vec);
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            None 
        ).unwrap();
        stream.play().unwrap();
        stream
    }

    fn musical_key(numerical_key: i32) -> String{
        return match numerical_key{
            1 => String::from("A"),
            2 => String::from("A#"),
            3 => String::from("B"),
            4 => String::from("C"),
            5 => String::from("C#"),
            6 => String::from("D"),
            7 => String::from("D#"),
            8 => String::from("E"),
            9 => String::from("F"),
            10 => String::from("F#"),
            11 => String::from("G"),
            12 => String::from("G#"),
            _ => String::from("???")
        };
        
    }

    fn calculate_key_and_octave(frequency: f32)->(i32,i32){
        let note_approximation: f32 = 12.*(frequency/440.).log2()+49.;
        let nearest_absolute_note: i32 = note_approximation.round() as i32;
        let octave: i32 = (nearest_absolute_note + 8) / 12;
        let numerical_key: i32 = nearest_absolute_note % 12;
        return (numerical_key, octave)
    }

    fn format_note(numerical_key: i32, octave: i32)->String{
        let key = Self::musical_key(numerical_key);
        return format!("{}_{}", key, octave)
    }

    fn cents_offset_by_numerical_key(n: i32, frequency: f32)->f32{
        return 1200. * (frequency / (f32::powf(2., (n as f32 - 49.) / 12. ) * 440.)).log2();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_cases_between_octaves(){
        let test_cases = [
            (27.5, "A_0"),
            (30.86771, "B_0"),
            (32.70320, "C_1"),
        ];
        for (input, expected) in test_cases {
            let (key, octave) = NoteDetector::calculate_key_and_octave(input);
            let result = NoteDetector::format_note(key, octave);
            assert_eq!(
                result, 
                expected, 
                "Test failed with input: {}", input
            );
        }
    }

    #[test]
    fn test_frequency_c_middle() {
        let freq = 261.63;
        let (key, octave) = NoteDetector::calculate_key_and_octave(freq);
        let note_str = NoteDetector::format_note(key, octave);
        
        assert_eq!(octave, 4, "Octave should be 4");
        assert_eq!(key, 4, "Key should be 4");
        assert_eq!(note_str, "C_4");
    }

    #[test]
    fn test_offset_calculation() {
        let offset = NoteDetector::cents_offset_by_numerical_key(49, 441.0);
        assert!(offset.abs() < 5., "cents_offset distance was {}. It should be less than 5", offset);
        assert!(offset.is_sign_positive(), "cents_offset should be positive when the frequency is above the target");
    }
}
