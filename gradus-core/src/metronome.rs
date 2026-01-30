use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{f32::consts::PI, sync::mpsc::{Receiver, Sender}};


#[derive(Debug, Clone)]
pub enum BeatType {
    Accent,
    SoftAccent,
    Normal,
    Silence,
}

impl BeatType {
    pub fn to_volume(&self) -> f32 {
        match self {
            BeatType::Accent => 1.0,
            BeatType::SoftAccent=> 0.7,
            BeatType::Normal => 0.4,
            BeatType::Silence => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RhythmPattern {
    pub steps: Vec<BeatType>,
}

impl RhythmPattern {
    pub fn default_from_signature() -> Self {
        let mut steps = Vec::with_capacity(4 as usize);
        for i in 0..4 {
            if i == 0 {
                steps.push(BeatType::Accent);
            } else {
                steps.push(BeatType::Normal);
            }
        }
        Self { steps }
    }
}

pub struct MetronomeSynth {
    pub is_playing: bool,
    sample_rate: f32,
    frequency: f32,
    master_volume: f32,
    volume: f32,
    samples_per_beat: u32,
    current_sample_count: u32,
    beep_duration: u32,
    rhythm_pattern: RhythmPattern,
    current_rhythm_index: usize,
    event_sender: Option<Sender<MetronomeEvent>>,
}

impl MetronomeSynth {
    pub fn new(sample_rate: f32, bpm: u32, master_volume: f32, beat_pattern: RhythmPattern, event_sender: Option<Sender<MetronomeEvent>>) -> Self {
        Self {
            sample_rate,
            frequency: 1000.0,
            master_volume,
            volume: 0.5,
            samples_per_beat: (sample_rate * 60.0 / bpm as f32) as u32,
            current_sample_count: 0,
            beep_duration: (sample_rate * 0.1) as u32, 
            is_playing: false,
            rhythm_pattern: beat_pattern,
            current_rhythm_index: 0,
            event_sender,
        }
    }

    pub fn set_rhythm_pattern(&mut self, new_pattern: RhythmPattern) {
        self.rhythm_pattern = new_pattern;
        self.current_rhythm_index = 0; 
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
                value = (t * self.frequency * 2.0 * PI).sin() * self.volume;
                let progress = self.current_sample_count as f32 / self.beep_duration as f32;
                value *= 1.0 - progress; 
            }
            for sample in frame.iter_mut() {
                *sample = value;
            }
            self.current_sample_count += 1;
            if self.current_sample_count >= self.samples_per_beat {
                self.current_sample_count = 0;
                self.current_rhythm_index += 1;
                if self.current_rhythm_index >= self.rhythm_pattern.steps.len() {
                    self.current_rhythm_index = 0;
                }
                if let Some(step_type) = self.rhythm_pattern.steps.get(self.current_rhythm_index) {
                    let volume_factor = step_type.to_volume();
                    self.volume = self.master_volume * volume_factor;
                }
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(MetronomeEvent::Tick(self.current_rhythm_index));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Metronome{
    pub is_playing: bool,
    rhythm_pattern: RhythmPattern,
    event_sender: Option<Sender<MetronomeEvent>>,
}

pub enum MetronomeCmd {
    SetBPM(u32),
    SetRhythmPattern(RhythmPattern),
    Play,
    Stop,
}

pub enum MetronomeEvent {
    Tick(usize)
}

impl Metronome{
    pub fn new(rhythm_pattern: RhythmPattern, sender: Sender<MetronomeEvent>) -> Self {
        Self {
            is_playing: false,
            rhythm_pattern: rhythm_pattern,
            event_sender:  Some(sender),
        }
    }

    pub fn run(self, cmd_receiver: Receiver<MetronomeCmd>) -> cpal::Stream {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let mut synth = MetronomeSynth::new(sample_rate as f32, 90, 0.5, self.rhythm_pattern, self.event_sender);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Ok(cmd) = cmd_receiver.try_recv() {
                    match cmd {
                        MetronomeCmd::SetBPM(bpm) => synth.set_bpm(bpm),
                        MetronomeCmd::SetRhythmPattern(pattern) => synth.set_rhythm_pattern(pattern),
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
