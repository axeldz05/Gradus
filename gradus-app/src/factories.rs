use relm4::{factory::{FactoryComponent, FactorySender}, prelude::DynamicIndex};
use gtk::prelude::*;
use gradus_core::beat::BeatAccent;

#[derive(Debug)]
pub struct BeatItem {
    pub state: Option<BeatAccent>,
}

#[derive(Debug)]
pub enum BeatInput {
    NextAccent,
    PreviousAccent,
}
#[derive(Debug)]
pub enum BeatOutput {
    Clicked,
}

impl BeatItem {
    fn next_accent(&mut self){
        match &mut self.state {
            Some(beat) => self.state = match beat {
                BeatAccent::Strong => None,
                BeatAccent::SoftStrong => Some(BeatAccent::Strong),
                BeatAccent::Weak => Some(BeatAccent::SoftStrong),
            },
            None => self.state = Some(BeatAccent::Weak),
        }
    }

    pub fn previous_accent(&mut self){
        match &mut self.state {
            Some(beat) => self.state = match beat {
                BeatAccent::Strong => Some(BeatAccent::SoftStrong),
                BeatAccent::SoftStrong => Some(BeatAccent::Weak),
                BeatAccent::Weak => None,
            },
            None => self.state = Some(BeatAccent::Strong),
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
            set_css_classes: match self.state {
                Some(BeatAccent::Weak) => &["beat-btn", "beat-weak"],
                Some(BeatAccent::SoftStrong) => &["beat-btn", "beat-soft-strong"],
                Some(BeatAccent::Strong) => &["beat-btn", "beat-strong"],
                None => &["beat-btn", "beat-none"],
            },
            connect_clicked[sender] => move |_| {
                sender.input(BeatInput::NextAccent);
                sender.output(BeatOutput::Clicked);
            },
            add_controller = gtk::GestureClick {
                set_button: gtk::gdk::BUTTON_SECONDARY,
                connect_pressed[sender] => move |gesture, _n_press, _x, _y| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    sender.input(BeatInput::PreviousAccent);
                    sender.output(BeatOutput::Clicked);
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { state: init }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            BeatInput::NextAccent => {
                self.next_accent();
            },
            BeatInput::PreviousAccent => {
                self.previous_accent();
            }
        }
    }
}
