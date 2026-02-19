use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use relm4::factory::FactoryVecDeque;
use gradus_core::beat::BeatAccent;
use gradus_core::metronome::{MetronomeCmd};
use std::sync::mpsc::Sender;
use crate::factories::{BeatInput, BeatItem, BeatOutput};

#[derive(Debug)]
pub enum BarEditorOutput {
    BarChanged(Vec<Option<BeatAccent>>)
}

pub struct BarEditorModel {
    beats: FactoryVecDeque<BeatItem>,
    audio_sender: Sender<MetronomeCmd>,
    error_msg: Option<String>,
}

#[derive(Debug)]
pub enum BarEditorMsg {
    BeatClicked,
    ClearError,
    SetAmountOfBeats(u32),
    TickReceived(usize)
}

#[relm4::component(pub)]
impl SimpleComponent for BarEditorModel {
    type Init = Sender<MetronomeCmd>;
    type Input = BarEditorMsg;
    type Output = BarEditorOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 10,
            gtk::Label {
                set_label: "Beats:",
                set_halign: gtk::Align::Start,
                set_css_classes: &["title-4"],
            },
            #[name = "entry_beats"]
            gtk::Entry{
                set_input_purpose: gtk::InputPurpose::Digits,
                connect_activate[sender] => move |entry| {
                    let text = entry.text();
                    if let Ok(amount_of_beats) = text.parse::<u32>() {
                        sender.input(BarEditorMsg::SetAmountOfBeats(amount_of_beats));
                    }
                }
            },
            #[local_ref]
            beat_flowbox -> gtk::FlowBox {
                set_valign: gtk::Align::Start,
                set_halign: gtk::Align::Fill,
                set_selection_mode: gtk::SelectionMode::None,
                set_min_children_per_line: 4,
                set_max_children_per_line: 8,
                set_row_spacing: 4,
                set_column_spacing: 4,
            },

            #[name = "error_label"]
            gtk::Label {
                #[watch]
                set_visible: model.error_msg.is_some(),
                #[watch]
                set_label: model.error_msg.as_deref().unwrap_or(""),
                set_css_classes: &["error"],
            }
        }
    }

    fn init(audio_sender: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let mut beats_factory = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::new())
            .forward(sender.input_sender(), |output| match output {
                BeatOutput::Clicked => BarEditorMsg::BeatClicked,
            });
        {
            let mut guard = beats_factory.guard();
            guard.push_back(Some(BeatAccent::Strong));
            guard.push_back(None);
            guard.push_back(None);
            guard.push_back(None);
            for _ in 1..4 {
            guard.push_back(Some(BeatAccent::Weak));
            guard.push_back(None);
            guard.push_back(None);
            guard.push_back(None);
            }
        }
        let model = BarEditorModel {
            beats: beats_factory,
            audio_sender,
            error_msg: None,
        };

        let beat_flowbox = model.beats.widget();

        let widgets = view_output!();
        widgets.entry_beats.set_text(&(model.beats.len()/4).to_string());
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BarEditorMsg::BeatClicked => {
                self.sync_audio(&sender);
            },
            BarEditorMsg::TickReceived(idx) => {
                self.beats.send(idx, BeatInput::HasTicked);
                let previous_idx = idx.checked_sub(1).unwrap_or(self.beats.len()-1);
                self.beats.send(previous_idx, BeatInput::EndTick);
            },
            BarEditorMsg::ClearError => self.error_msg = None,
            BarEditorMsg::SetAmountOfBeats(amount) => {
                self.set_amount_of_beats(amount);
                self.sync_audio(&sender);
            },
        }
    }
}

impl BarEditorModel {
    fn sync_audio(&mut self, sender: &ComponentSender<BarEditorModel>) {
        let beat_items : Vec<Option<BeatAccent>> = self.beats.guard().iter().map(|item| item.accent.into()).collect();
        let res = self.audio_sender.send(MetronomeCmd::SetBar(beat_items.clone()));
        match res {
            Ok(_) => (),
            Err(e) => println!("Error while setting bar in metronome Synth: {}", e)
        }
        let res = sender.output(BarEditorOutput::BarChanged(beat_items));
        match res {
            Ok(_) => (),
            Err(e) => println!("Error while setting bar in metronome UI: {:?}", e)
        }
    }

    pub fn set_amount_of_beats(&mut self, amount_of_beats: u32) {
        // this assumes self.beats to be divisible by 4
        // this may become a problem if numbers are too big, which shouldn't be the case
        let beat_diff = ((amount_of_beats*4) as i32 - self.beats.len() as i32)/4;
        let mut guard = self.beats.guard();
        println!("beat_diff: {}", beat_diff);
        if beat_diff > 0 {
            for _ in 0..beat_diff {
                guard.push_back(Some(BeatAccent::SoftStrong));
                guard.push_back(None);
                guard.push_back(None);
                guard.push_back(None);
            }
        } else if beat_diff < 0 {
            for _ in 0..-beat_diff {
                guard.pop_back();
                guard.pop_back();
                guard.pop_back();
                guard.pop_back();
            }
        }
        // todo!("Make beats based on whether it's a quarter, whole, etc.");
    }
}

