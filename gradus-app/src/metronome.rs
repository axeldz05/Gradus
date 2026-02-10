use std::sync::mpsc;

use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent, ComponentController};
use gradus_core::{editor::{BeatAccent, Bar}, metronome::{Metronome, MetronomeCmd}};

use crate::bar_editor::{BarEditorModel, BarEditorOutput};

pub struct MetronomeModel {
    active: bool,
    engine_sender: std::sync::mpsc::Sender<MetronomeCmd>, 
    current_beat_index: Option<usize>,
    bar: Bar,
    bar_editor: relm4::Controller<BarEditorModel>,
    bar_editor_active: bool,
    #[allow(dead_code)]
    _stream: Option<cpal::Stream>,
}

#[derive(Debug)]
pub enum MetronomeMsg {
    ToggleActive,
    TickReceived(usize),
    SetBar(Bar),
    ToggleMeasureEditor
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
                set_label: &format!("Beat: {:?}. Time signature: {}/{}", &model.current_beat_index, &model.bar.time_signature_upper, &model.bar.time_signature_lower),
                set_css_classes: &["title-1"],
            },
            #[name = "beats_canvas"]
            gtk::DrawingArea {
                set_content_height: 50,
                set_content_width: 300,
                #[watch]
                set_draw_func: {
                    let pattern = model.bar.clone();
                    let current_index = model.current_beat_index;
                    move |_, context, w, h| {
                        let step_count = pattern.beats.len();
                        let padding = 10.0;
                        let box_size = (w as f64 - (padding * (step_count as f64 - 1.0))) / step_count as f64;
                        for (i, step) in pattern.beats.iter().enumerate() {
                            let x = i as f64 * (box_size + padding);
                            let y = (h as f64 - box_size) / 2.0;
                            if let Some(current_step) = step{
                                let (r, g, b) = match current_step {
                                    BeatAccent::Strong => (0.9, 0.3, 0.3),
                                    BeatAccent::Weak => (0.3, 0.3, 0.9),
                                    _ => (0.5, 0.5, 0.5),
                                };
                                let is_active = Some(i) == current_index;
                                if is_active {
                                    context.set_source_rgb(r, g, b);
                                    context.rectangle(x, y, box_size, box_size);
                                    context.fill().expect("fill failed");
                                    context.set_source_rgb(1.0, 1.0, 1.0);
                                    context.set_line_width(2.0);
                                    context.rectangle(x + 2.0, y + 2.0, box_size - 4.0, box_size - 4.0);
                                    context.stroke().expect("stroke failed");
                                } else {
                                    context.set_source_rgba(r, g, b, 0.3);
                                    context.rectangle(x, y, box_size, box_size);
                                    context.fill().expect("fill dim failed");
                                }
                            }
                        }
                    }
                }
            },
            #[name = "editor"]
            gtk::Expander {
                set_label: match model.bar_editor_active {
                    true => Some("Close Editor"),
                    false => Some("Edit Pattern"),
                },
                set_expanded: model.bar_editor_active,
                connect_expanded_notify[sender] => move |_expander| {
                    sender.input(MetronomeMsg::ToggleMeasureEditor);
                },
                set_child: Some(model.bar_editor.widget()),
            },
            gtk::Button{
               connect_clicked => MetronomeMsg::ToggleActive,
               set_label: &format!("{}", if model.active {"Stop"} else {"Start"})
            }
        }
    }

    fn init(_: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let (tx, rx) = mpsc::channel::<MetronomeCmd>();
        let (event_tx, event_rx) = mpsc::channel();
        let metronome = Metronome::new(Bar::default_from_signature(), event_tx);
        let stream = metronome.run(rx);
        let sender_clone = sender.clone();
        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                match event {
                    gradus_core::metronome::MetronomeEvent::Tick(idx) => {
                        sender_clone.input(MetronomeMsg::TickReceived(idx));
                    }
                }
            }
        });

        let bar_editor = BarEditorModel::builder()
            .launch(tx.clone())
            .forward(sender.input_sender(), |msg| match msg {
                BarEditorOutput::BarChanged(bar) => MetronomeMsg::SetBar(bar)
            });

        let model = MetronomeModel {
            active:  false,
            engine_sender: tx,
            bar: Bar::default_from_signature(),
            current_beat_index: Some(0),
            bar_editor,
            bar_editor_active: false,
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
            MetronomeMsg::TickReceived(idx) => {
                self.current_beat_index = Some(idx);
            },
            MetronomeMsg::ToggleMeasureEditor => {
                self.bar_editor_active = !self.bar_editor_active;
            },
            MetronomeMsg::SetBar(bar) => {
                self.bar = bar;
            },
        }
    }
}
