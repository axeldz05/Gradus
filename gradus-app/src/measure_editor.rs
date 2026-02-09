use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use gradus_core::editor::{Measure, EditorNote, NoteDuration, NoteAccent};
use gradus_core::metronome::{MetronomeCmd};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum MeasureEditorOutput {
    MeasureChanged(Measure)
}

pub struct MeasureEditorModel {
    measure: Measure,
    selected_duration: NoteDuration,
    selected_accent: NoteAccent,
    audio_sender: Sender<MetronomeCmd>,
    error_msg: Option<String>,
    editor_width: i32,
    editor_height: i32
}

#[derive(Debug)]
pub enum MeasureEditorMsg {
    SetTool(NoteDuration, NoteAccent),
    CanvasClick { x: f64, y: f64, button: u32, width: f32 },
    ChangeSignature { upper: u32, lower: u32 },
    ClearError,
}

#[relm4::component(pub)]
impl SimpleComponent for MeasureEditorModel {
    type Init = Sender<MetronomeCmd>;
    type Input = MeasureEditorMsg;
    type Output = MeasureEditorOutput;

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
                gtk::Button {
                    set_label: "Quarter note (Strong)",
                    connect_clicked => MeasureEditorMsg::SetTool(NoteDuration::Quarter, NoteAccent::Strong),
                },
                gtk::Button {
                    set_label: "Eighth note (Normal)",
                    connect_clicked => MeasureEditorMsg::SetTool(NoteDuration::Eighth, NoteAccent::Weak),
                },
                gtk::Button {
                    set_label: "Sixteenth note",
                    connect_clicked => MeasureEditorMsg::SetTool(NoteDuration::Sixteenth, NoteAccent::Weak),
                },
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
                        sender.input(MeasureEditorMsg::CanvasClick { x, y, button, width});
                    }
                },

                #[watch]
                set_draw_func: {
                    let measure = model.measure.clone();
                    let total_ticks = measure.total_ticks();

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

                        for i in 0..measure.notes.len() {
                            let x = i as f64 * tick_width;
                            if let Some(note) = measure.notes[i]{
                                let width = note.duration as u32 as f64 * tick_width;
                                let (r, g, b) = match note.accent {
                                    NoteAccent::Strong => (0.8, 0.2, 0.2),
                                    NoteAccent::SoftStrong => (0.55, 0.2, 0.2),
                                    NoteAccent::Weak => (0.2, 0.4, 0.8),
                                    NoteAccent::Mute => (0.5, 0.5, 0.5),
                                };

                            // body of the note
                            context.set_source_rgb(r, g, b);
                            context.rectangle(x + 1.0, 10.0, width - 2.0, h as f64 - 20.0);
                            context.fill().expect("Fill note failed");

                            // border
                            context.set_source_rgb(0.0, 0.0, 0.0);
                            context.set_line_width(2.0);
                            context.rectangle(x + 1.0, 10.0, width - 2.0, h as f64 - 20.0);
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
        let model = MeasureEditorModel {
            measure: Measure::new(4, 4),
            selected_duration: NoteDuration::Quarter,
            selected_accent: NoteAccent::Strong,
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
            MeasureEditorMsg::SetTool(dur, acc) => {
                self.selected_duration = dur;
                self.selected_accent = acc;
            }
            
            MeasureEditorMsg::CanvasClick { x, y, button, width } => {
                println!("CanvasClick. X: {}, Y: {}, button: {}", x, y, button);
                let total_ticks = self.measure.total_ticks() as usize;
                let tick_width = width as f64 / total_ticks as f64;
                println!("total_ticks: {}, tick_width: {}", total_ticks, tick_width);
                let clicked_index = (x / tick_width).floor() as usize;
                if clicked_index < total_ticks {
                    println!("at position: {}", clicked_index);
                    if button == 1 {
                        let res = self.measure.set_note(EditorNote{
                            duration: self.selected_duration, 
                            accent: self.selected_accent}, clicked_index);
                        match res {
                            Err(error) => {
                                println!("set_note error: {}", error)
                            },
                            Ok(_) => ()
                        }
                        self.sync_audio(&sender);
                    } else if button == 3 {
                        let res = self.measure.remove_note(clicked_index);
                        match res {
                            Err(error) => {
                                println!("remove_note error: {}", error)
                            },
                            Ok(_) => ()
                        }
                        self.sync_audio(&sender);
                    }
                }
            }

            MeasureEditorMsg::ChangeSignature { upper, lower } => {
                self.measure = Measure::new(upper, lower);
                self.sync_audio(&sender);
            }
            
            MeasureEditorMsg::ClearError => self.error_msg = None,
        }
    }
}

impl MeasureEditorModel {
    fn sync_audio(&self, sender: &ComponentSender<MeasureEditorModel>) {
        let res = self.audio_sender.send(MetronomeCmd::SetMeasure(self.measure.clone()));
        match res {
            Ok(_) => (),
            Err(e) => println!("Error while setting measure in metronome Synth: {}", e)
        }
        let res = sender.output(MeasureEditorOutput::MeasureChanged(self.measure.clone()));
        match res {
            Ok(_) => (),
            Err(e) => println!("Error while setting measure in metronome UI: {:?}", e)
        }
    }
}

