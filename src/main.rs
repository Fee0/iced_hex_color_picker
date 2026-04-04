use hex_color::{ColorMap, Rgb};
use iced::widget::text;
use iced::{Element, Theme};
use iced_color_map::editor::{self, ColorMapEditor, Event};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("Color Map Editor")
        .theme(Theme::Dark)
        .centered()
        .run()
}

struct App {
    editor: ColorMapEditor,
    status: String,
}

impl App {
    fn boot() -> Self {
        let initial = ColorMap::new([Rgb::from_hex(0x202020); 256]);
        Self {
            editor: ColorMapEditor::new(&initial),
            status: String::new(),
        }
    }

    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::Editor(msg) => {
                if let Some(event) = self.editor.update(msg) {
                    match event {
                        Event::Accepted(_map) => {
                            self.status = "Color map accepted.".into();
                        }
                        Event::Cancelled => {
                            self.status = "Editing cancelled.".into();
                        }
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<AppMessage> {
        let editor = self.editor.view().map(AppMessage::Editor);

        if self.status.is_empty() {
            editor
        } else {
            iced::widget::column![editor, text(&self.status).size(14)]
                .spacing(8)
                .into()
        }
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    Editor(editor::Message),
}
