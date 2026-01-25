use std::sync::mpsc::Sender;
use pitch_detection::detector::{PitchDetector, mcleod::McLeodDetector};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug)]
pub struct NoteDetectorDaemon{
    pub frequency: f32,
    pub note: String,
}

impl NoteDetectorDaemon{

    pub fn connect(sender: Sender<NoteDetectorDaemon>) -> cpal::Stream {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No se encontró micrófono");
        let config = device.default_input_config().unwrap();

        let sample_rate = config.sample_rate() as usize;

        const WINDOW_SIZE: usize = 1024;
        const PADDING: usize = WINDOW_SIZE / 2;
        const POWER_THRESHOLD: f32 = 5.0;
        const CLARITY_THRESHOLD: f32 = 0.6;

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut detector = McLeodDetector::new(WINDOW_SIZE, PADDING);
                let mut buffer: Vec<f32> = Vec::with_capacity(WINDOW_SIZE);
                for &sample in data {
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
                        let frequency: f32 = pitch.frequency as f32;
                        let nota = NoteDetectorDaemon {
                            frequency: frequency,
                            note: Self::determinate_note(frequency)
                        };

                        let _ = sender.send(nota);
                    }
                    buffer.clear(); 
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            None 
        ).unwrap();
        stream.play().unwrap();
        return stream
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

    fn determinate_note(frequency: f32)->String{
        let note_approximation: f32 = 12.*(frequency/440.).log2()+49.;
        let nearest_absolute_note: i32 = (note_approximation.round() as i32) + 9;
        let octave: i32 = nearest_absolute_note / 12;
        let numerical_key: i32 = nearest_absolute_note % 12;
        let key = Self::musical_key(numerical_key);
        return format!("{}_{}", key, octave)
    }
}
