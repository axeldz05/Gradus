use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use verovioxide::{Options, Svg, Toolkit};

pub struct ScoreEditorModel {
    buffer: gtk::TextBuffer,
    current_svg_path: String,
    error_msg: Option<String>,
}

#[derive(Debug)]
pub enum ScoreEditorMsg {
    UpdateScore,
}

#[relm4::component(pub)]
impl SimpleComponent for ScoreEditorModel {
    type Init = ();
    type Input = ScoreEditorMsg;
    type Output = ();

    view! {
        gtk::Paned {
            set_orientation: gtk::Orientation::Horizontal,
            set_hexpand: true,
            set_vexpand: true,
            set_position: 400,

            #[wrap(Some)]
            set_start_child =  &gtk::Frame{
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        gtk::Label {
                            #[watch]
                            set_visible: model.error_msg.is_some(),
                            #[watch]
                            set_label: model.error_msg.as_deref().unwrap_or(""),
                            set_css_classes: &["error"],
                        },

                        #[name = "score_image"]
                        gtk::Picture {
                            set_can_shrink: true, 
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Start,
                            #[watch]
                            set_filename: if model.current_svg_path.is_empty() { None } else { Some(model.current_svg_path.as_str()) },
                        }
                    }
                }
            },

            #[wrap(Some)]
            set_end_child = &gtk::Frame{
                gtk::ScrolledWindow {
                    gtk::TextView {
                        set_buffer: Some(&model.buffer),
                        set_monospace: true,
                        set_wrap_mode: gtk::WrapMode::WordChar,
                        set_top_margin: 10,
                        set_bottom_margin: 10,
                        set_left_margin: 10,
                        set_right_margin: 10,
                    }
                }
            }
        }
    }

    fn init(_: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let buffer = gtk::TextBuffer::new(None);
        let default_abc = "X:1\nT:My Score\nM:4/4\nL:1/4\nK:C\nC D E F | G A B c |";
        buffer.set_text(default_abc);
        let sender_clone = sender.clone();
        buffer.connect_changed(move |_| {
            sender_clone.input(ScoreEditorMsg::UpdateScore);
        });
        let sender_clone2 = sender.clone();
        buffer.connect_cursor_position_notify(move |_| {
            sender_clone2.input(ScoreEditorMsg::UpdateScore);
        });

        let model = ScoreEditorModel {
            buffer,
            current_svg_path: String::new(),
            error_msg: None,
        };
        let widgets = view_output!();
        sender.input(ScoreEditorMsg::UpdateScore);
        
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ScoreEditorMsg::UpdateScore => {
                let start = self.buffer.start_iter();
                let end = self.buffer.end_iter();
                let text = self.buffer.text(&start, &end, false).to_string();
                let mut voxide = Toolkit::new().expect("Error Voxide");
                let options = Options::builder()
                    .input_from("abc")
                    .scale(70)
                    .page_width(800)
                    .page_margin_left(20)
                    .page_margin_right(20)
                    .page_margin_top(20)
                    .page_margin_bottom(20)
                    .adjust_page_height(true)
                    .build();
                voxide.set_options(&options).expect("Error options");
                if voxide.load_data(&text).is_ok() {
                    if let Ok(base_svg) = voxide.render(Svg::page(1)) { 
                        let (target_measure_index, target_note_index) = self.find_cursor_position_in_abc_score();
                        let doc = match roxmltree::Document::parse(&base_svg) {
                            Ok(d) => d,
                            Err(e) => {
                                eprintln!("Error while trying to parse with roxmltree: {}", e);
                                return; 
                            }
                        };
                        let mut measure_nodes = doc.descendants().filter(|n| {
                            n.attribute("class")
                                .map(|classes| classes.split_whitespace().any(|c| c == "measure"))
                                .unwrap_or(false)
                        });
                        if let Some(measure_node) = measure_nodes.nth(target_measure_index) {
                            let mut note_nodes = measure_node.descendants().filter(|n| {
                                n.attribute("class")
                                    .map(|classes| classes.split_whitespace().any(|c| c == "note"))
                                    .unwrap_or(false)
                            });
                            if let Some(note_node) = note_nodes.nth(target_note_index - 1) {
                                let svg_final = match note_node.attribute("id") {
                                    Some(note_id) => Self::highlight_note_in_svg(&base_svg, note_id),
                                    _ => base_svg
                                };
                                let temp_path = "/tmp/gradus_score.svg";
                                std::fs::write(temp_path, &svg_final).unwrap();
                                self.current_svg_path = temp_path.to_string();
                                self.error_msg = None;
                            }
                        }
                    }
                } else {
                    self.error_msg = Some("Invalid ABC syntax or is incomplete...".to_string());
                }
            }
        }
    }
}

impl ScoreEditorModel {
    fn highlight_note_in_svg(base_svg: &str, note_id: &str) -> String {
        let style_tag = format!(
            "<style>
            #{id}, #{id} * {{
                fill: #ff0000 !important;
                stroke: #ff0000 !important;
            }}
        </style>",
        id = note_id
        );
        base_svg.replace("</svg>", &format!("{}\n</svg>", style_tag))
    }

    fn find_cursor_position_in_abc_score(&self) -> (usize, usize) {
        let cursor_position = self.buffer.cursor_position();
        let mut end_iter = self.buffer.start_iter();
        end_iter.forward_cursor_positions(cursor_position);
        let binding = self.buffer.text(&self.buffer.start_iter(), &end_iter, false);
        let text = binding.as_str();
        let target_measure_index = text.matches('|').count();
        let text_after_last_bar = match text.rfind('|') {
            Some(index) => &text[index + 1..],
            None => text.lines().last().unwrap_or("")
        };
        let mut target_note_index = text_after_last_bar.split_whitespace().count();
        if target_note_index == 0 {
            target_note_index = 1;
        }
        (target_measure_index, target_note_index)
    }
}
