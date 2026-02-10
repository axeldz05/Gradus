use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BeatAccent {
    Strong,
    SoftStrong,
    Weak,
}

impl BeatAccent {
    pub fn to_volume(&self) -> f32 {
        match self {
            BeatAccent::Strong => 1.0,
            BeatAccent::SoftStrong => 0.65,
            BeatAccent::Weak => 0.4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub time_signature_upper: u32,
    pub time_signature_lower: u32,
    pub beats: Vec<Option<BeatAccent>>,
}

impl Bar {
    pub fn default_from_signature() -> Self {
        Self { time_signature_upper: 4, 
            time_signature_lower: 4, 
            beats: vec![Some(BeatAccent::Strong), 
            None,
            None,
            None,
            Some(BeatAccent::Weak),
            None,
            None,
            None,
            Some(BeatAccent::Weak),
            None,
            None,
            None,
            Some(BeatAccent::Weak),
            None,
            None,
            None,
            ] 
        }
    }
    pub fn new(upper: u32, lower: u32) -> Self {
        let ticks = (upper * 16) / lower;
        Self {
            time_signature_upper: upper,
            time_signature_lower: lower,
            beats: vec![None; ticks as usize]
        }
    }

    pub fn next_beat_accent(&mut self, at_pos: usize) -> Result<(), String> {
        if at_pos >= self.beats.len() {
            return Err(format!("Position {} out of bounds", at_pos));
        }
        match &mut self.beats[at_pos] {
            Some(beat) => self.beats[at_pos] = match beat {
                BeatAccent::Strong => None,
                BeatAccent::SoftStrong => Some(BeatAccent::Strong),
                BeatAccent::Weak => Some(BeatAccent::SoftStrong),
            },
            None => self.beats[at_pos] = Some(BeatAccent::Weak),
        }
        Ok(())
    }

    pub fn previous_beat_accent(&mut self, at_pos: usize) -> Result<(), String> {
        if at_pos >= self.beats.len() {
            return Err(format!("Position {} out of bounds", at_pos));
        }
        match &mut self.beats[at_pos] {
            Some(beat) => self.beats[at_pos] = match beat {
                BeatAccent::Strong => Some(BeatAccent::SoftStrong),
                BeatAccent::SoftStrong => Some(BeatAccent::Weak),
                BeatAccent::Weak => None,
            },
            None => self.beats[at_pos] = Some(BeatAccent::Strong),
        }
        Ok(())
    }
    
    pub fn total_ticks(&self) -> u32 {
        // Assumes base resolution of sixteenth beat (1/16)
        (self.time_signature_upper * 16) / self.time_signature_lower
    }
}
