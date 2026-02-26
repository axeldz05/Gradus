use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use verovioxide::{Toolkit, Options};

pub struct PartitureEditorModel {
    current_svg_path: String,
    time_signature: String, 
    key_signature: String,  
    default_length: String, 
    selected_duration: String, 
    dots: u8,           
    current_accidental: String,
    events: Vec<String>, 
}

#[derive(Debug)]
pub enum PartitureEditorMsg {
    SetKey(String),
    SetTime(String),
    SetDefaultLength(String),
    DecreaseDuration,
    IncreaseDuration,
    SetDots(u8),
    SetAccidental(String),
    AddPitch(String),
    AddRest,
    AddBarline,
    Undo,
    RenderScore,
}

#[allow(deprecated)]
#[relm4::component(pub)]
impl SimpleComponent for PartitureEditorModel {
    type Init = ();
    type Input = PartitureEditorMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            set_spacing: 10,
            set_css_classes: &["partiture-box"],
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                gtk::Label { set_label: "Tone:" },
                gtk::ComboBoxText {
                    append_text: "C", append_text: "G", append_text: "F",
                    set_active: Some(0),
                    connect_changed[sender] => move |combo| {
                        if let Some(text) = combo.active_text() {
                            sender.input(PartitureEditorMsg::SetKey(text.to_string()));
                        }
                    }
                },
                gtk::Label { set_label: " Meter:" },
                gtk::ComboBoxText {
                    append_text: "4/4", append_text: "3/4", append_text: "6/8",
                    set_active: Some(0),
                    connect_changed[sender] => move |combo| {
                        if let Some(text) = combo.active_text() {
                            sender.input(PartitureEditorMsg::SetTime(text.to_string()));
                        }
                    }
                },
                gtk::Label { set_label: " Base:" },
                gtk::ComboBoxText {
                    append_text: "1/4", append_text: "1/8",
                    set_active: Some(0),
                    set_tooltip_text: Some("Unit note length"),
                    connect_changed[sender] => move |combo| {
                        if let Some(text) = combo.active_text() {
                            sender.input(PartitureEditorMsg::SetDefaultLength(text.to_string()));
                        }
                    }
                },
            },
            set_can_focus: true,
            add_controller = gtk::EventControllerKey {
                connect_key_pressed[sender] => move |_, key, _, _modifier| {
                    use gtk::gdk::Key;
                    let mut handled = true;
                    match key {
                        Key::_1 | Key::KP_1 => sender.input(PartitureEditorMsg::AddPitch("C".to_string())),
                        Key::_2 | Key::KP_2 => sender.input(PartitureEditorMsg::AddPitch("D".to_string())),
                        Key::_3 | Key::KP_3 => sender.input(PartitureEditorMsg::AddPitch("E".to_string())),
                        Key::_4 | Key::KP_4 => sender.input(PartitureEditorMsg::AddPitch("F".to_string())),
                        Key::_5 | Key::KP_5 => sender.input(PartitureEditorMsg::AddPitch("G".to_string())),
                        Key::_6 | Key::KP_6 => sender.input(PartitureEditorMsg::AddPitch("A".to_string())),
                        Key::_7 | Key::KP_7 => sender.input(PartitureEditorMsg::AddPitch("B".to_string())),

                        Key::s | Key::S => sender.input(PartitureEditorMsg::SetAccidental("^".to_string())),
                        Key::b | Key::B => sender.input(PartitureEditorMsg::SetAccidental("_".to_string())),

                        Key::period => sender.input(PartitureEditorMsg::SetDots(1)),
                        Key::colon => sender.input(PartitureEditorMsg::SetDots(2)),

                        Key::r | Key::R => sender.input(PartitureEditorMsg::AddRest),
                        Key::Return | Key::KP_Enter => sender.input(PartitureEditorMsg::AddBarline),
                        
                        Key::minus => sender.input(PartitureEditorMsg::DecreaseDuration),
                        Key::plus => sender.input(PartitureEditorMsg::IncreaseDuration),

                        Key::Undo => sender.input(PartitureEditorMsg::Undo),
                        _ => handled = false,
                    }

                    // Le decimos a GTK si consumimos el evento o si debe pasarlo a otros widgets
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
            },
            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                
                #[name = "score_image"]
                gtk::Picture {
                    set_can_shrink: false, 
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    #[watch]
                    set_filename: if model.current_svg_path.is_empty() { None } else { Some(model.current_svg_path.as_str()) },
                }
            }
        }
    }

    fn init(_: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = PartitureEditorModel {
            current_svg_path: String::new(),
            time_signature: "4/4".to_string(),
            key_signature: "C".to_string(),
            default_length: "1/4".to_string(),
            selected_duration: "".to_string(), 
            dots: 0,
            current_accidental: String::new(),
            events: vec![],
        };
        let widgets = view_output!();
        sender.input(PartitureEditorMsg::RenderScore);
        
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PartitureEditorMsg::SetKey(k) => { self.key_signature = k; sender.input(PartitureEditorMsg::RenderScore); }
            PartitureEditorMsg::SetTime(t) => { self.time_signature = t; sender.input(PartitureEditorMsg::RenderScore); }
            PartitureEditorMsg::SetDefaultLength(l) => { self.default_length = l; sender.input(PartitureEditorMsg::RenderScore); }
            PartitureEditorMsg::DecreaseDuration => {
                self.selected_duration = self.selected_duration + "/";
            },
            PartitureEditorMsg::IncreaseDuration => {
                self.selected_duration = self.selected_duration + "/";
            },
            PartitureEditorMsg::SetDots(n) => self.dots = n,
            PartitureEditorMsg::SetAccidental(acc) => self.current_accidental = acc,
            PartitureEditorMsg::AddPitch(pitch) => {
                let dot_str = ">".repeat(self.dots as usize);
                let acc = self.current_accidental.clone();
                let modifier = format!("{}{}", self.selected_duration, dot_str);
                let token = format!("{}{}{}", acc, pitch, modifier);
                self.current_accidental.clear();
                self.dots = 0;

                let incoming_duration = self.parse_duration_token(&modifier);
                let current_used = self.current_measure_duration();
                let capacity = self.get_measure_capacity();
                if current_used + incoming_duration > capacity + 0.01 {
                    self.events.push("|".to_string());
                }
                self.events.push(token);
                sender.input(PartitureEditorMsg::RenderScore);
            }
            PartitureEditorMsg::AddRest => {
                let dot_str = ">".repeat(self.dots as usize);
                let token = format!("z{}{}", self.selected_duration, dot_str);
                self.events.push(token);
                sender.input(PartitureEditorMsg::RenderScore);
            }
            PartitureEditorMsg::AddBarline => {
                self.events.push("|".to_string());
                sender.input(PartitureEditorMsg::RenderScore);
            }
            PartitureEditorMsg::Undo => {
                self.events.pop();
                sender.input(PartitureEditorMsg::RenderScore);
            }
            PartitureEditorMsg::RenderScore => {
                let header = format!(
                    "X:1\nM:{}\nL:{}\nK:{}\n", 
                    self.time_signature, self.default_length, self.key_signature
                );
                
                // beaming algorithm
                let mut music_body = String::new();
                let mut current_beat_duration = 0.0;
                let beam_max = self.get_beam_group_duration();

                for event in &self.events {
                    if event == "|" {
                        music_body.push_str(" | ");
                        current_beat_duration = 0.0;
                    } else if event.starts_with('z') {
                        music_body.push_str(event);
                        music_body.push(' ');
                        current_beat_duration = 0.0;
                    } else {
                        let modifier = Self::extract_modifier(event);
                        let duration = self.parse_duration_token(&modifier);
                        if duration >= 1.0 {
                            music_body.push_str(event);
                            music_body.push(' ');
                            current_beat_duration = 0.0;
                        } else if current_beat_duration + duration > beam_max + 0.01 {
                            music_body.push(' '); 
                            music_body.push_str(event);
                            current_beat_duration = duration;
                        } else {
                            music_body.push_str(event);
                            current_beat_duration += duration;
                            if current_beat_duration >= beam_max - 0.01 {
                                music_body.push(' ');
                                current_beat_duration = 0.0;
                            }
                        }
                    }
                }

                // ABC Notation needs a | at the end to be able to render
                if self.events.last() != Some(&"|".to_string()){
                    music_body.push_str(" |");
                }
                let abc_data = format!("{}{}", header, music_body);
                let mut voxide = Toolkit::new().expect("Error Voxide");
                let options = Options::builder()
                    .input_from("abc")
                    .scale(50)
                    .adjust_page_height(true)
                    .build();
                voxide.set_options(&options).expect("Error while setting voxide options");
                voxide.load_data(&abc_data).expect("Error while loading abc_data");
                if voxide.page_count() > 0 { 
                    let svg_string = voxide.render_to_svg(1).expect("Error while rendering to svg");
                    let temp_path = "/tmp/gradus_score.svg";
                    std::fs::write(temp_path, &svg_string).expect("Error while writing svg_string to temp_path");
                    self.current_svg_path = temp_path.to_string();
                }
            }
        }
    }
}
impl PartitureEditorModel {
    fn get_base_duration(&self) -> f32 {
        match self.default_length.as_str() {
            "1/4" => 1.0,
            "1/8" => 0.5,
            "1/16" => 0.25,
            _ => 1.0,
        }
    }

    fn parse_duration_token(&self, token_modifier: &str) -> f32 {
        let base = self.get_base_duration();
        token_modifier.chars().fold(base, |acc, c| match c {
            '1'..='9' => acc * c.to_digit(10).unwrap() as f32,
            '/' => acc / 2.0,
            '.' | '>' => acc * 1.5,
            _ => acc, 
        })
    }

    fn extract_modifier(event: &str) -> String {
        let chars: Vec<char> = event.chars().collect();
        chars[1..].iter().collect()
    }

    fn current_measure_duration(&self) -> f32 {
        let mut current_duration = 0.0;
        for event in self.events.iter().rev() {
            if event == "|" {
                break;
            }
            let modifier = Self::extract_modifier(event);
            current_duration += self.parse_duration_token(&modifier);
        }
        current_duration
    }

    fn get_measure_capacity(&self) -> f32 {
        match self.time_signature.as_str() {
            "4/4" => 4.0,
            "3/4" => 3.0,
            "6/8" => 3.0,
            _ => 4.0,
        }
    }
    fn get_beam_group_duration(&self) -> f32 {
        match self.time_signature.as_str() {
            "6/8" | "9/8" | "12/8" => 1.5,
            _ => 1.0,
        }
    }
}
