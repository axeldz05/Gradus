use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent};
use gtk::*;
use notedetector::NoteDetector;
use std::thread;
use std::sync::mpsc;

pub struct TunerModel {
    current_note: String,
    freq: f32,
    offset: f32,
    #[allow(dead_code)]
    _stream: Option<cpal::Stream>,
}

#[derive(Debug)]
pub enum TunerMsg {
    UpdateNote(String, f32, f32),
}

#[relm4::component(pub)]
impl SimpleComponent for TunerModel {
    type Init = ();
    type Input = TunerMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            gtk::Label {
                #[watch]
                set_label: &format!("Note: {}. Offset: {}", &model.current_note, &model.offset),
                set_css_classes: &["title-1"],
            }
        }
    }

    fn init(_: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let (tx, rx) = mpsc::channel::<NoteDetector>();

        let stream = NoteDetector::connect(tx);
        let sender_clone = sender.clone();
        
        thread::spawn(move || {
            while let Ok(data) = rx.recv() {
                sender_clone.input(TunerMsg::UpdateNote(data.note, data.frequency, data.offset));
            }
        });

        let model = TunerModel {
            current_note: String::from("--"),
            freq: 0.,
            offset: 0.,
            _stream: Some(stream)
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            TunerMsg::UpdateNote(note, freq, offset) => {
                self.current_note = note;
                self.freq = freq;
                self.offset = offset;
            }
        }
    }
}
