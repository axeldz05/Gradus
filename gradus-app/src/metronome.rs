use std::{f64::consts::PI, sync::mpsc, time::Instant};

use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent, ComponentController};
use gradus_core::{beat::BeatAccent, metronome::{Metronome, MetronomeCmd}};

use crate::bar_editor::{BarEditorModel, BarEditorMsg, BarEditorOutput};

const SWING_AMPLITUDE: f64 = PI / 4.0;
const STOP_ANIMATION_DURATION: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    Idle,
    Running,
    Stopping,
}

pub struct MetronomeModel {
    active: bool,
    engine_sender: std::sync::mpsc::Sender<MetronomeCmd>, 
    current_beat_index: Option<usize>,
    bar: Vec<Option<BeatAccent>>,
    bar_editor: relm4::Controller<BarEditorModel>,
    bar_editor_active: bool,
    #[allow(dead_code)]
    _stream: Option<cpal::Stream>,
    start_time: Option<Instant>,
    stop_start_time: Option<Instant>,
    angle_on_stop: f64,
    state: AnimationState,
    bpm: u32
}

#[derive(Debug)]
pub enum MetronomeMsg {
    ToggleActive,
    TickReceived(usize),
    SetBar(Vec<Option<BeatAccent>>),
    ToggleMeasureEditor,
    AnimationTick(Instant),
    UpdateBpm(u32)
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
            #[name = "metronome"]
            gtk::DrawingArea {
                set_content_height: 200,
                set_content_width: 300,
                set_hexpand: true,
                set_vexpand: true,
                #[watch]
                set_draw_func: {
                    let state = model.state;
                    let bpm = model.bpm;

                    let start_time = model.start_time;
                    let stop_start_time = model.stop_start_time;
                    let angle_on_stop = model.angle_on_stop;

                    move |_, context, w, h| {
                        let center_x = w as f64 / 2.0;
                        let bottom_y = h as f64 - 20.0;
                        let length = h as f64 - 50.0;

                        let now = Instant::now();
                        
                        let angle = match state {
                            AnimationState::Idle => 0.0,
                            AnimationState::Running => {
                                if let Some(start) = start_time {
                                    let elapsed = now.duration_since(start).as_secs_f64();
                                    (elapsed * (bpm as f64 / 60.0) * PI).sin() * SWING_AMPLITUDE
                                } else {
                                    0.0
                                }
                            },
                            AnimationState::Stopping => {
                                if let Some(stop_start) = stop_start_time {
                                    let elapsed_ms = now.duration_since(stop_start).as_millis() as f64;
                                    let duration_ms = STOP_ANIMATION_DURATION as f64;
                                    let t = (elapsed_ms / duration_ms).min(1.0);
                                    angle_on_stop * (1.0 - t)
                                } else {
                                    0.0
                                }
                            }
                        };

                        let tip_x = center_x + length * angle.sin();
                        let tip_y = bottom_y - length * angle.cos();

                        let end_angle = PI/4.;
                        // background
                        context.rectangle(0.,0., 450., 200.);
                        context.set_source_rgba(0.4, 0.4, 0.4, 0.69);
                        context.fill().expect("Fill failed");

                        // center of the pendulum
                        context.set_source_rgba(1.,1.,1., 0.69);
                        context.set_line_width(3.);
                        context.set_line_cap(gtk::cairo::LineCap::Square);
                        context.move_to(center_x, bottom_y);
                        context.line_to(center_x, bottom_y - length * 0.75);
                        context.stroke().expect("Stroke failed");
                        // right end of the pendulum
                        context.set_source_rgba(1.,1.,1., 0.33);
                        context.set_line_width(1.5);
                        context.set_line_cap(gtk::cairo::LineCap::Square);
                        context.move_to(center_x, bottom_y);
                        context.line_to(center_x + length * end_angle.cos(), bottom_y - length * end_angle.sin());
                        context.stroke().expect("Stroke failed");
                        // left end of the pendulum
                        context.set_source_rgba(1.,1.,1., 0.33);
                        context.set_line_width(1.5);
                        context.set_line_cap(gtk::cairo::LineCap::Square);
                        context.move_to(center_x, bottom_y);
                        context.line_to(center_x - length * end_angle.cos(), bottom_y - length * end_angle.sin());
                        context.stroke().expect("Stroke failed");

                        // pendulum
                        context.set_source_rgb(0.2, 0.2, 0.2);
                        context.set_line_width(6.0);
                        context.set_line_cap(gtk::cairo::LineCap::Round);
                        context.move_to(center_x, bottom_y);
                        context.line_to(tip_x, tip_y);
                        context.stroke().expect("Stroke failed");
                        
                        // tip of the pendulum
                        context.set_source_rgb(0.85, 0.3, 0.3);
                        context.arc(tip_x, tip_y, 5.0, 0.0, 2.0 * PI);
                        context.fill().expect("Fill failed");
                        
                        // base of the pendulum
                        context.set_source_rgb(0.1, 0.1, 0.1);
                        context.arc(center_x, bottom_y, 8.0, 0.0, 2.0 * PI);
                        context.fill().expect("Fill failed");
                    }
                }
            },
            #[name = "bpm_entry"]
            gtk::Entry{
                set_input_purpose: gtk::InputPurpose::Digits,
                connect_activate[sender] => move |entry| {
                    let text = entry.text();
                    if let Ok(new_bpm) = text.parse::<u32>() {
                        sender.input(MetronomeMsg::UpdateBpm(new_bpm));
                    }
                }
            },
            #[name = "editor"]
            gtk::Box {
                append: model.bar_editor.widget(),
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
        let metronome = Metronome::new(event_tx);
        let bar = vec![Some(BeatAccent::Strong), 
            None,
            None,
            None,
            Some(BeatAccent::Weak),
            None,
            None,
            None,
            Some(BeatAccent::Weak),
            None,
            None,
            None,
            Some(BeatAccent::Weak),
            None,
            None,
            None,
            ]; 
        let stream = metronome.run(bar.clone(), rx);
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
            bar: bar,
            current_beat_index: Some(0),
            bar_editor,
            bar_editor_active: false,
            bpm: 60,
            state: AnimationState::Idle,
            start_time: None,
            stop_start_time: None,
            angle_on_stop: 0.0,
            _stream: Some(stream)
        };
        model.engine_sender.send(MetronomeCmd::SetBPM(model.bpm)).expect("Failure to set BPM at start");

        let widgets = view_output!();
        widgets.metronome.add_tick_callback(move |_, _clock| {
            sender.input(MetronomeMsg::AnimationTick(Instant::now()));
            glib::ControlFlow::Continue
        });
        widgets.bpm_entry.set_text(&model.bpm.to_string());
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            MetronomeMsg::ToggleActive => {
                self.active = !self.active;
                if self.engine_sender.send(if self.active {MetronomeCmd::Play} else {MetronomeCmd::Stop}).is_err(){
                    panic!("aaaaa");
                }
                match self.state {
                    AnimationState::Idle | AnimationState::Stopping => {
                        self.state = AnimationState::Running;
                        self.start_time = Some(Instant::now());
                    },
                    AnimationState::Running => {
                        self.state = AnimationState::Stopping;
                        self.stop_start_time = Some(Instant::now());
                        if let Some(start) = self.start_time {
                             let elapsed = Instant::now().duration_since(start).as_secs_f64();
                             self.angle_on_stop = (elapsed * (self.bpm as f64 / 60.0) * PI).sin() * SWING_AMPLITUDE;
                        }
                    }
                }
            },
            MetronomeMsg::TickReceived(idx) => {
                self.current_beat_index = Some(idx);
                if let Err(err) = self.bar_editor.sender().send(BarEditorMsg::TickReceived(idx)){
                    println!("Error while trying to update tick: {:?}", err);
                }
            },
            MetronomeMsg::ToggleMeasureEditor => {
                self.bar_editor_active = !self.bar_editor_active;
            },
            MetronomeMsg::SetBar(bar) => {
                self.bar = bar;
            },
            MetronomeMsg::AnimationTick(_now) => {
                match self.state {
                    AnimationState::Running => {
                        return;
                    },
                    AnimationState::Stopping => {
                        if let Some(start) = self.stop_start_time {
                            if start.elapsed().as_millis() as u64 > STOP_ANIMATION_DURATION {
                                self.state = AnimationState::Idle;
                                self.angle_on_stop = 0.0;
                            }
                        }
                    },
                    AnimationState::Idle => {
                        return; 
                    }
                }
            },
            MetronomeMsg::UpdateBpm(new_bpm) => {
                self.bpm = new_bpm;
                self.engine_sender.send(MetronomeCmd::SetBPM(new_bpm)).expect("Setting BPM failure");
            }
        }
    }
}
