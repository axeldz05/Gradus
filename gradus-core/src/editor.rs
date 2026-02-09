use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NoteDuration {
    Whole = 16,
    Half = 8,
    Quarter = 4,
    Eighth = 2,
    Sixteenth = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NoteAccent {
    Strong,
    SoftStrong,
    Weak,
    Mute,
}

impl NoteAccent {
    pub fn to_volume(&self) -> f32 {
        match self {
            NoteAccent::Strong => 1.0,
            NoteAccent::SoftStrong => 0.65,
            NoteAccent::Weak => 0.4,
            NoteAccent::Mute => 0.0,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct EditorNote {
    pub start_pos: u32,
    pub duration: NoteDuration,
    pub accent: NoteAccent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measure {
    pub time_signature_upper: u32,
    pub time_signature_lower: u32,
    pub notes: Vec<EditorNote>,
}

impl Measure {
    pub fn default_from_signature() -> Self {
        Self { time_signature_upper: 4, 
            time_signature_lower: 4, 
            notes: vec![EditorNote {
                start_pos: 0, 
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Strong }, 
            EditorNote {
                start_pos: 4, 
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Weak },
            EditorNote {
                start_pos: 8, 
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Weak },
            EditorNote {
                start_pos: 12, 
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Weak },

            ] 
        }
    }
    pub fn new(upper: u32, lower: u32) -> Self {
        Self {
            time_signature_upper: upper,
            time_signature_lower: lower,
            notes: Vec::new(),
        }
    }

    pub fn try_add_note(&mut self, new_note: EditorNote) -> Result<(), String> {
        let total_capacity = self.total_ticks();
        if new_note.start_pos + (new_note.duration as u32) > total_capacity {
            return Err(
                format!("Note exceeds the duration available in the measure at pos: {}", 
                    new_note.start_pos));
        }
        for note in &self.notes {
            if self.is_overlapping(note, &new_note) {
                return Err(format!("There's a note overlapping at pos: {}", 
                    new_note.start_pos));
            }
        }
        self.notes.push(new_note);
        self.notes.sort_by_key(|n| n.start_pos);
        Ok(())
    }
    
    pub fn total_ticks(&self) -> u32 {
        // Assumes base resolution of sixteenth note (1/16)
        (self.time_signature_upper * 16) / self.time_signature_lower
    }
    
    pub fn is_overlapping(&self, n1: &EditorNote, n2: &EditorNote) -> bool {
        let n1_end = n1.start_pos + n1.duration as u32;
        let n2_end = n2.start_pos + n2.duration as u32;
        n1.start_pos < n2_end && n2.start_pos < n1_end
    }
}
