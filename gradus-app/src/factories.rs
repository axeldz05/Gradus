use relm4::{factory::{FactoryComponent, FactorySender}, prelude::DynamicIndex};
use gtk::prelude::*;
use gradus_core::beat::BeatAccent;

#[derive(Debug)]
pub struct BeatItem {
    pub accent: Option<BeatAccent>,
    has_ticked: bool,
}

#[derive(Debug)]
pub enum BeatInput {
    NextAccent,
    PreviousAccent,
    HasTicked,
    EndTick,
}
#[derive(Debug)]
pub enum BeatOutput {
    Clicked,
}

impl BeatItem {
    fn next_accent(&mut self){
        match &mut self.accent {
            Some(beat) => self.accent = match beat {
                BeatAccent::Strong => None,
                BeatAccent::SoftStrong => Some(BeatAccent::Strong),
                BeatAccent::Weak => Some(BeatAccent::SoftStrong),
            },
            None => self.accent = Some(BeatAccent::Weak),
        }
    }

    pub fn previous_accent(&mut self){
        match &mut self.accent {
            Some(beat) => self.accent = match beat {
                BeatAccent::Strong => Some(BeatAccent::SoftStrong),
                BeatAccent::SoftStrong => Some(BeatAccent::Weak),
                BeatAccent::Weak => None,
            },
            None => self.accent = Some(BeatAccent::Strong),
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for BeatItem {
    type Init = Option<BeatAccent>;
    type Input = BeatInput;
    type Output = BeatOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Button {
            add_css_class: "beat-btn",
            set_width_request: 30,
            set_height_request: 30,
            
            #[watch]
            set_css_classes: &[
                "beat-btn",
                match self.accent {
                    Some(BeatAccent::Strong) => "beat-strong",
                    Some(BeatAccent::SoftStrong) => "beat-soft-strong",
                    Some(BeatAccent::Weak) => "beat-weak",
                    _ => "beat-none",
                },
                if self.has_ticked { "beat-tick" } else { "" },
            ],
            connect_clicked[sender] => move |_| {
                sender.input(BeatInput::NextAccent);
                if let Err(err) = sender.output(BeatOutput::Clicked){
                    println!("Error while sending BeatOutput::Clicked: {:?}", err);
                }
            },
            add_controller = gtk::GestureClick {
                set_button: gtk::gdk::BUTTON_SECONDARY,
                connect_pressed[sender] => move |gesture, _n_press, _x, _y| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    sender.input(BeatInput::PreviousAccent);
                    if let Err(err) = sender.output(BeatOutput::Clicked){
                        println!("Error while sending BeatOutput::Clicked: {:?}", err);
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { accent: init, has_ticked: false }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            BeatInput::NextAccent => {
                self.next_accent();
            },
            BeatInput::PreviousAccent => {
                self.previous_accent();
            }
            BeatInput::HasTicked => {
                self.has_ticked = true;
            },
            BeatInput::EndTick => {
                self.has_ticked = false;
            }
        }
    }
}
