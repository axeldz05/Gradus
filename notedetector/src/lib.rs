use std::{sync::mpsc::{self, Sender}, thread};
use pitch_detection::detector::{PitchDetector, mcleod::McLeodDetector};
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
            const POWER_THRESHOLD: f32 = 5.0;
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
                            offset: Self::offset_by_numerical_key(key, frequency)
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
            1 => String::from("C"),
            2 => String::from("C#"),
            3 => String::from("D"),
            4 => String::from("D#"),
            5 => String::from("E"),
            6 => String::from("F"),
            7 => String::from("F#"),
            8 => String::from("G"),
            9 => String::from("G#"),
            10 => String::from("A"),
            11 => String::from("A#"),
            12 => String::from("B"),
            _ => String::from("???")
        };
        
    }

    fn calculate_key_and_octave(frequency: f32)->(i32,i32){
        let note_approximation: f32 = 12.*(frequency/440.).log2()+49.;
        let nearest_absolute_note: i32 = (note_approximation.round() as i32) + 9;
        let octave: i32 = nearest_absolute_note / 12;
        let numerical_key: i32 = nearest_absolute_note % 12;
        return (numerical_key, octave)
    }

    fn format_note(numerical_key: i32, octave: i32)->String{
        let key = Self::musical_key(numerical_key);
        return format!("{}_{}", key, octave)
    }

    fn offset_by_numerical_key(n: i32, frequency: f32)->f32{
        return (f32::powf(2., (n as f32 - 49.) / 12. ) * 440.) - frequency/440.;
    }
}
