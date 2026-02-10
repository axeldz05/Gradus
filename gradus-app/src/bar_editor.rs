use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use gradus_core::editor::{Bar, BeatAccent};
use gradus_core::metronome::{MetronomeCmd};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum BarEditorOutput {
    BarChanged(Bar)
}

pub struct BarEditorModel {
    bar: Bar,
    audio_sender: Sender<MetronomeCmd>,
    error_msg: Option<String>,
    editor_width: i32,
    editor_height: i32
}

#[derive(Debug)]
pub enum BarEditorMsg {
    CanvasClick { x: f64, y: f64, button: u32, width: f32 },
    ChangeSignature { upper: u32, lower: u32 },
    ClearError,
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
                set_label: "Editor tools:",
                set_halign: gtk::Align::Start,
                set_css_classes: &["title-4"],
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
            },

            #[name = "editor_canvas"]
            gtk::DrawingArea {
                set_content_height: model.editor_height,
                set_content_width: model.editor_width,
                set_hexpand: false,
                set_vexpand: false,
                set_halign: gtk::Align::Center,
                add_controller = gtk::GestureClick {
                    set_button: 0, // 0 = listen to all buttons (left and right)
                    connect_pressed[sender] => move |gesture, _, x, y| {
                        let button = gesture.current_button();
                        let widget = gesture.widget().expect("Gesture must be tied to a widget");
                        let width = widget.width() as f32;
                        sender.input(BarEditorMsg::CanvasClick { x, y, button, width});
                    }
                },

                #[watch]
                set_draw_func: {
                    let bar = model.bar.clone();
                    let total_ticks = bar.total_ticks();

                    move |_, context, w, h| {
                        let tick_width = w as f64 / total_ticks as f64;
                        context.set_source_rgb(0.9, 0.9, 0.9);
                        context.paint().expect("Paint failed");
                        context.set_source_rgb(0.7, 0.7, 0.7);
                        context.set_line_width(1.0);
                        for i in 0..=total_ticks{
                            let x = i as f64 * tick_width;
                            context.move_to(x, 0.0);
                            context.line_to(x, h as f64);
                        }
                        context.stroke().expect("Stroke failed");

                        for i in 0..bar.beats.len() {
                            let x = i as f64 * tick_width;
                            if let Some(note) = bar.beats[i]{
                                let (r, g, b) = match note {
                                    BeatAccent::Strong => (0.8, 0.2, 0.2),
                                    BeatAccent::SoftStrong => (0.55, 0.2, 0.2),
                                    BeatAccent::Weak => (0.2, 0.4, 0.8),
                                };

                            // body of the note
                            context.set_source_rgb(r, g, b);
                            context.rectangle(x + 1.0, 10.0, tick_width - 2.0, h as f64 - 20.0);
                            context.fill().expect("Fill note failed");

                            // border
                            context.set_source_rgb(0.0, 0.0, 0.0);
                            context.set_line_width(2.0);
                            context.rectangle(x + 1.0, 10.0, tick_width - 2.0, h as f64 - 20.0);
                            context.stroke().expect("Stroke note failed");
                            }
                        }
                    }
                }
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
        let model = BarEditorModel {
            bar: Bar::new(4, 4),
            audio_sender,
            error_msg: None,
            editor_width: 400,
            editor_height: 200
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BarEditorMsg::CanvasClick { x, y, button, width } => {
                println!("CanvasClick. X: {}, Y: {}, button: {}", x, y, button);
                let total_ticks = self.bar.total_ticks() as usize;
                let tick_width = width as f64 / total_ticks as f64;
                println!("total_ticks: {}, tick_width: {}", total_ticks, tick_width);
                let clicked_index = (x / tick_width).floor() as usize;
                if clicked_index < total_ticks {
                    println!("at position: {}", clicked_index);
                    if button == 1 {
                        let res = self.bar.next_beat_accent(clicked_index);
                        match res {
                            Err(error) => {
                                println!("next_beat_accent error: {}", error)
                            },
                            Ok(_) => ()
                        }
                        self.sync_audio(&sender);
                    } else if button == 3 {
                        let res = self.bar.previous_beat_accent(clicked_index);
                        match res {
                            Err(error) => {
                                println!("previous_beat_accent error: {}", error)
                            },
                            Ok(_) => ()
                        }
                        self.sync_audio(&sender);
                    }
                }
            }

            BarEditorMsg::ChangeSignature { upper, lower } => {
                self.bar = Bar::new(upper, lower);
                self.sync_audio(&sender);
            }
            
            BarEditorMsg::ClearError => self.error_msg = None,
        }
    }
}

impl BarEditorModel {
    fn sync_audio(&self, sender: &ComponentSender<BarEditorModel>) {
        let res = self.audio_sender.send(MetronomeCmd::SetMeasure(self.bar.clone()));
        match res {
            Ok(_) => (),
            Err(e) => println!("Error while setting bar in metronome Synth: {}", e)
        }
        let res = sender.output(BarEditorOutput::BarChanged(self.bar.clone()));
        match res {
            Ok(_) => (),
            Err(e) => println!("Error while setting bar in metronome UI: {:?}", e)
        }
    }
}

