use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{f32::consts::PI, sync::mpsc::{Receiver, Sender}};

use crate::editor::{Measure, NoteDuration};

pub struct MetronomeSynth {
    pub is_playing: bool,
    sample_rate: f32,
    frequency: f32,
    master_volume: f32,
    volume: f32,
    samples_per_beat: u32,
    current_sample_count: f32,
    beep_duration: u32,
    measure: Measure,
    current_rhythm_index: usize,
    event_sender: Option<Sender<MetronomeEvent>>,
}

impl MetronomeSynth {
    pub fn new(sample_rate: f32, bpm: u32, master_volume: f32, measure: Measure, event_sender: Option<Sender<MetronomeEvent>>) -> Self {
        Self {
            sample_rate,
            frequency: 1000.0,
            master_volume,
            volume: 0.5,
            samples_per_beat: (sample_rate * 60.0 / bpm as f32) as u32,
            current_sample_count: 0.,
            beep_duration: (sample_rate * 0.1) as u32, 
            is_playing: false,
            measure: measure,
            current_rhythm_index: 0,
            event_sender,
        }
    }

    pub fn set_measure(&mut self, new_measure: Measure) {
        self.measure = new_measure;
        self.current_rhythm_index = 0; 
    }

    pub fn set_bpm(&mut self, bpm: u32) {
        if bpm > 0 {
            self.samples_per_beat = 
                (self.sample_rate * 60.0 / (bpm as u32) as f32) as u32;
        }
    }

    pub fn process(&mut self, output: &mut [f32], channels: usize) {
        if !self.is_playing {
            output.fill(0.0);
            return;
        }
        for frame in output.chunks_mut(channels) {
            let mut value = 0.0;
            if (self.current_sample_count as u32) < self.beep_duration {
                let t = self.current_sample_count as f32 / self.sample_rate;
                value = (t * self.frequency * 2.0 * PI).sin() * self.volume;
                let progress = self.current_sample_count as f32 / self.beep_duration as f32;
                value *= 1.0 - progress; 
            }
            for sample in frame.iter_mut() {
                *sample = value;
            }
            match self.measure.notes[self.current_rhythm_index].duration {
                NoteDuration::Whole=> self.current_sample_count += 0.25,
                NoteDuration::Half => self.current_sample_count += 0.5,
                NoteDuration::Quarter => self.current_sample_count += 1.,
                NoteDuration::Eighth => self.current_sample_count += 2.,
                NoteDuration::Sixteenth => self.current_sample_count += 4.,
            }
            if self.current_sample_count as u32 >= self.samples_per_beat {
                self.current_sample_count = 0.;
                self.current_rhythm_index += 1;
                if self.current_rhythm_index >= self.measure.notes.len() {
                    self.current_rhythm_index = 0;
                }
                if let Some(step_type) = self.measure.notes.get(self.current_rhythm_index) {
                    let volume_factor = step_type.accent.to_volume();
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
    measure: Measure,
    event_sender: Option<Sender<MetronomeEvent>>,
}

pub enum MetronomeCmd {
    SetBPM(u32),
    SetMeasure(Measure),
    Play,
    Stop,
}

pub enum MetronomeEvent {
    Tick(usize)
}

impl Metronome{
    pub fn new(measure: Measure, sender: Sender<MetronomeEvent>) -> Self {
        Self {
            is_playing: false,
            measure,
            event_sender:  Some(sender),
        }
    }

    pub fn run(self, cmd_receiver: Receiver<MetronomeCmd>) -> cpal::Stream {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let mut synth = MetronomeSynth::new(sample_rate as f32, 60, 0.5, self.measure, self.event_sender);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Ok(cmd) = cmd_receiver.try_recv() {
                    match cmd {
                        MetronomeCmd::SetBPM(bpm) => synth.set_bpm(bpm),
                        MetronomeCmd::SetMeasure(pattern) => synth.set_measure(pattern),
                        MetronomeCmd::Play => synth.is_playing = true,
                        MetronomeCmd::Stop => {
                            synth.is_playing = false;
                            synth.current_sample_count = 0.;
                        },
                    }
                }
                synth.process(data, channels);
            },
            |err| eprintln!("Metronome error: {}", err),
            None
        ).unwrap();

        stream.play().unwrap();
        stream
    }
}
