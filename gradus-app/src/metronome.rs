use std::sync::mpsc;

use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent};
use gradus_core::metronome::{MetronomeCmd, Metronome};

pub struct MetronomeModel {
    active: bool,
    bpm: u32,
    engine_sender: std::sync::mpsc::Sender<MetronomeCmd>, 
//    tick_receiver: std::sync::mpsc::Receiver<()>,
    #[allow(dead_code)]
    _stream: Option<cpal::Stream>,
}

#[derive(Debug)]
pub enum MetronomeMsg {
    ToggleActive
}

#[relm4::component(pub)]
impl SimpleComponent for MetronomeModel {
    type Init = ();
    type Input = MetronomeMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            gtk::Label {
                #[watch]
                // set_label: &format!("Beat: {}. Time signature: {}/{}", &model.current_step, &model.notes_in_a_measure,&model.note_type),
                set_css_classes: &["title-1"],
            },
            gtk::Button{
               connect_clicked => MetronomeMsg::ToggleActive,
               set_label: &format!("{}", if model.active {"Stop"} else {"Start"})
            }
        }
    }

    fn init(_: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let (tx, rx) = mpsc::channel::<MetronomeCmd>();
        let metronome = Metronome::new(gradus_core::metronome::RhythmPattern::default_from_signature());
        let stream = metronome.run(rx);
        let model = MetronomeModel {
            active:  false,
            bpm: 90,
            engine_sender: tx,
            _stream: Some(stream)
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
                MetronomeMsg::ToggleActive => {
                self.active = !self.active;
                if self.engine_sender.send(if self.active {MetronomeCmd::Play} else {MetronomeCmd::Stop}).is_err(){
                    panic!("aaaaa");
                }
            },
        }
    }
}
