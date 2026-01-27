use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt, SimpleComponent};

use gtk::prelude::*;

use crate::header::{HeaderModel, HeaderOutput};
use crate::dialog::{DialogModel, DialogOutput, DialogInput};

use crate::tuner::TunerModel;
use crate::metronome::MetronomeModel;

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Tuner,
}
pub struct AppModel {
    mode: AppMode,
    header: relm4::Controller<HeaderModel>,
    dialog: relm4::Controller<DialogModel>,
    tuner: relm4::Controller<TunerModel>,
    metronome: relm4::Controller<MetronomeModel>,
}

#[derive(Debug)]
pub enum AppMsg {
    SetMode(AppMode),
    CloseRequest,
    Close,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = AppMode;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[root]
        gtk::ApplicationWindow::builder()
            .titlebar(&gtk::HeaderBar::new())
            .default_width(500)
            .default_height(250)
            .title("Gradus")
            .build() {
                set_titlebar: Some(model.header.widget()),
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,
                    #[name = "root_stack"]
                    gtk::Stack {
                        set_vexpand: false,
                        set_vhomogeneous: false,

                        add_named[Some("tuner")] = model.tuner.widget(),
                        add_named[Some("metronome")] = model.metronome.widget(),
                    },
                },
                connect_close_request[sender] => move |_| {
                    sender.input(AppMsg::CloseRequest);
                    gtk::glib::Propagation::Stop
                }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header: Controller<HeaderModel> =
            HeaderModel::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                HeaderOutput::Tuner => AppMsg::SetMode(AppMode::Tuner),
            });

        let dialog = DialogModel::builder()
            .transient_for(&root)
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                DialogOutput::Close => AppMsg::Close,
            });

        let tuner = TunerModel::builder()
            .launch(())
            .detach();

        let metronome = MetronomeModel::builder()
            .launch(())
            .detach();

        let model = AppModel {
            mode: params,
            header,
            dialog,
            tuner,
            metronome
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SetMode(mode) => {
                self.mode = mode;
            }
            AppMsg::CloseRequest => {
                self.dialog.sender().send(DialogInput::Show).unwrap();
            }
            AppMsg::Close => {
                relm4::main_application().quit();
            }
        }
    }
}

pub fn launch() {
    let app = RelmApp::new("relm4.test.simple");
    app.run::<AppModel>(AppMode::Tuner);
}
