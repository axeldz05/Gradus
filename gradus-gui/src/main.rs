mod tuner;
mod dialog;
mod header;

use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt, SimpleComponent, prelude::AsyncComponent};

use gtk::{
    glib::{self, clone, ControlFlow},
    prelude::{
        BoxExt, ButtonExt, Cast, FileChooserExt, FileExt, GtkWindowExt,
        OrientableExt, WidgetExt,
    },
    ApplicationWindow, ButtonsType, FileChooserAction, FileChooserDialog, MessageDialog,
    MessageType, ResponseType,
};

use header::{HeaderModel, HeaderOutput};
use dialog::{DialogModel, DialogOutput, DialogInput};

#[derive(Debug)]
enum AppMode {
    Tuner,
}
struct AppModel {
    mode: AppMode,
    header: relm4::Controller<HeaderModel>,
    dialog: relm4::Controller<DialogModel>,
}

#[derive(Debug)]
enum AppMsg {
    SetMode(AppMode),
    CloseRequest,
    Close,
}

// Idea:
// hay que hacer un drag-and-drop en gtk para cada componente llamado
// como el Tuner, Metronome, etc.
// La idea es que se puedan mover entre distintas posiciones.
// Estos componentes deben de poder escalar manualmente y automaticamente
// segun el espacio disponible. (Posiblemente con un max_widht y max_height)
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
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,

                    gtk::Label {

                        #[watch]
                        set_label: &format!("Placeholder for {:?}", model.mode),
                    },
                    #[local_ref]
                    tool_grid -> gtk::Grid {
                        set_orientation: gtk::Orientation::Vertical,
                        set_column_spacing: 15,
                        set_row_spacing: 5,
                    }

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

        let model = AppModel {
            mode: params,
            header,
            dialog,
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

fn main() {
    let app = RelmApp::new("relm4.test.simple");
    app.run::<AppModel>(AppMode::Tuner);
}
