use relm4::*;
use gtk::prelude::*;

use crate::app::{AppMsg, AppMode};


#[derive(Debug)]

pub struct HeaderModel;

#[relm4::component(pub)]
impl SimpleComponent for HeaderModel {
    type Init = ();
    type Input = ();
    type Output = AppMsg;

    view! {
        #[root]
        gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: relm4::css::LINKED,
                append: group = &gtk::ToggleButton {
                    set_label: "Tuner",
                    set_active: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Tuner)).unwrap()
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Metronome",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Metronome)).unwrap()
                        }
                    },
                },
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
