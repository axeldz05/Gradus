use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent};
use gtk::*;
use notedetector::NoteDetectorDaemon;

pub struct TunerModel {
    current_note: String,
}

#[derive(Debug)]
pub enum TunerMsg {
    UpdateNote(f32),
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
                set_label: &model.current_note,
                set_css_classes: &["title-1"],
            }
        }
    }

    fn init(_: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {

        let model = TunerModel {
            current_note: String::from("--"),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            TunerMsg::UpdateNote(note) => {
                self.current_note = note.to_string();
            }
        }
    }
}
