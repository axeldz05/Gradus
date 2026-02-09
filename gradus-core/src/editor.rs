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
    pub duration: NoteDuration,
    pub accent: NoteAccent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measure {
    pub time_signature_upper: u32,
    pub time_signature_lower: u32,
    pub notes: Vec<Option<EditorNote>>,
}

impl Measure {
    pub fn default_from_signature() -> Self {
        Self { time_signature_upper: 4, 
            time_signature_lower: 4, 
            notes: vec![Some(EditorNote {
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Strong }), 
            None,
            None,
            None,
            Some(EditorNote {
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Weak }),
            None,
            None,
            None,
            Some(EditorNote {
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Weak }),
            None,
            None,
            None,
            Some(EditorNote {
                duration: NoteDuration::Quarter,
                accent: NoteAccent::Weak }),
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
            notes: vec![None; ticks as usize]
        }
    }

    pub fn set_note(&mut self, new_note: EditorNote, at_pos: usize) -> Result<(), String> {
        if at_pos >= self.notes.len() {
            return Err(format!("Position {} out of bounds", at_pos));
        }
        if self.is_overlapping(&new_note, at_pos){
            return Err(format!("Overlap detected with note at pos {}", at_pos));
        }
        self.notes[at_pos] = Some(new_note);
        Ok(())
    }

    pub fn remove_note(&mut self, at_pos: usize) -> Result<(), String> {
        if at_pos >= self.notes.len() {
            return Err(format!("Position {} out of bounds", at_pos));
        }
        self.notes[at_pos] = None;
        Ok(())
    }
    
    pub fn total_ticks(&self) -> u32 {
        // Assumes base resolution of sixteenth note (1/16)
        (self.time_signature_upper * 16) / self.time_signature_lower
    }
    
    pub fn is_overlapping(&self, new_note: &EditorNote, new_note_pos: usize) -> bool {
        for (i, slot) in self.notes.iter().enumerate() {
            if let Some(existing_note) = slot {
                let n1_end = i + existing_note.duration as usize;
                let n2_end = new_note_pos + new_note.duration as usize;
                if i < n2_end && new_note_pos < n1_end {
                    return true
                }
            }
        }
        false
    }
}
