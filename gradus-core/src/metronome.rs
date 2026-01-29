use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{f32::consts::PI, sync::mpsc::Receiver};


pub struct MetronomeSynth {
    pub is_playing: bool,
    sample_rate: f32,
    frequency: f32,
    volume: f32,
    samples_per_beat: u32,
    current_sample_count: u32,
    beep_duration: u32,
    current_beat: u32,
    beats_in_a_measure: u32,
}

impl MetronomeSynth {
    pub fn new(sample_rate: f32, bpm: u32) -> Self {
        Self {
            sample_rate,
            frequency: 1000.0,
            volume: 0.5,
            samples_per_beat: (sample_rate * 60.0 / bpm as f32) as u32,
            current_sample_count: 0,
            beep_duration: (sample_rate * 0.1) as u32, 
            is_playing: false,
            current_beat: 1,
            beats_in_a_measure: 4,
        }
    }

    pub fn set_bpm(&mut self, bpm: u32) {
        if bpm > 0 {
            self.samples_per_beat = (self.sample_rate * 60.0 / bpm as f32) as u32;
        }
    }

    pub fn process(&mut self, output: &mut [f32], channels: usize) {
        if !self.is_playing {
            output.fill(0.0);
            return;
        }
        for frame in output.chunks_mut(channels) {
            let mut value = 0.0;
            if self.current_sample_count < self.beep_duration {
                let t = self.current_sample_count as f32 / self.sample_rate;
                if self.current_beat == 1 {
                    value = (t * self.frequency * 3.0 * PI).sin() * self.volume;
                } else {
                    value = (t * self.frequency * 2.0 * PI).sin() * self.volume;
                }
                let progress = self.current_sample_count as f32 / self.beep_duration as f32;
                value *= 1.0 - progress; 
            }
            for sample in frame.iter_mut() {
                *sample = value;
            }
            self.current_sample_count += 1;
            if self.current_sample_count >= self.samples_per_beat {
                self.current_sample_count = 0;
                if self.current_beat >= self.beats_in_a_measure{
                    self.current_beat = 1;
                } else{
                    self.current_beat += 1;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Metronome{
    pub is_playing: bool,
}

pub enum MetronomeCmd {
    SetBPM(u32),
    Play,
    Stop,
}

impl Metronome{
    pub fn new() -> Self {
        Self {
            is_playing: false,
        }
    }
    pub fn run(cmd_receiver: Receiver<MetronomeCmd>) -> cpal::Stream {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let mut synth = MetronomeSynth::new(sample_rate as f32, 90);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Ok(cmd) = cmd_receiver.try_recv() {
                    match cmd {
                        MetronomeCmd::SetBPM(bpm) => synth.set_bpm(bpm),
                        MetronomeCmd::Play => synth.is_playing = true,
                        MetronomeCmd::Stop => {
                            synth.is_playing = false;
                            synth.current_sample_count = 0;
                        },
                    }
                }
                synth.process(data, channels);
            },
            |err| eprintln!("Error en metrónomo: {}", err),
            None
        ).unwrap();

        stream.play().unwrap();
        stream
    }
}
