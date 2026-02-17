use serde::{Deserialize, Serialize};

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
            BeatAccent::SoftStrong => 0.5,
            BeatAccent::Weak => 0.25,
        }
    }
}
