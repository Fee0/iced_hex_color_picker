use hex_color::presets::{ascii_classes, AsciiClassColors};
use hex_color::Rgb;
use iced::alignment;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::text as w_text;
use iced::widget::{button, column, container, stack, text};
use iced::{clipboard, Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme};
use iced_color_map::editor::{self, ColorMapEditor, Event};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("Color Map Editor â€” Demo")
        .theme(Theme::Dark)
        .centered()
        .run()
}

struct App {
    editor: ColorMapEditor,
    accepted_map: [Rgb; 256],
    editing: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    OpenEditor,
    Editor(editor::Message),
}

impl App {
    fn boot() -> Self {
        let initial = ascii_classes(AsciiClassColors {
            null: Rgb::from_hex(0x404040),
            printable: Rgb::from_hex(0x00CC00),
            whitespace: Rgb::from_hex(0xCCCC00),
            control: Rgb::from_hex(0xCC0000),
            non_ascii: Rgb::from_hex(0x0066CC),
        });
        let table = *initial.as_table();
        Self {
            editor: ColorMapEditor::new(&initial),
            accepted_map: table,
            editing: false,
        }
    }

    fn update(&mut self, message: Msg) -> Task<Msg> {
        match message {
            Msg::OpenEditor => {
                self.editing = true;
                Task::none()
            }
            Msg::Editor(msg) => {
                if matches!(&msg, editor::Message::Picker(editor::PickerMessage::CopyHex)) {
                    clipboard::write::<Msg>(self.editor.picker_hex_string())
                } else if let Some(event) = self.editor.update(msg) {
                    match event {
                        Event::Accepted(map) => {
                            self.accepted_map = *map.as_table();
                            self.editing = false;
                        }
                        Event::Cancelled => {
                            self.editing = false;
                        }
                    }
                    Task::none()
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Msg> {
        let preview_grid = canvas(PreviewProgram {
            colors: &self.accepted_map,
        })
        .width(Length::Fixed(PREVIEW_SIDE))
        .height(Length::Fixed(PREVIEW_SIDE));

        let main_view = container(
            column![
                button(text("Edit Color Map").size(16))
                    .on_press(Msg::OpenEditor)
                    .padding(12),
                text("Current Color Map").size(14).font(iced::Font::MONOSPACE),
                preview_grid,
            ]
            .spacing(12)
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .padding(24);

        if self.editing {
            let backdrop = container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.6,
                    })),
                    ..Default::default()
                });

            let editor_panel = container(self.editor.view().map(Msg::Editor))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.15,
                        a: 1.0,
                    })),
                    border: Border {
                        color: Color {
                            r: 0.3,
                            g: 0.3,
                            b: 0.3,
                            a: 1.0,
                        },
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .padding(8);

            let overlay = container(editor_panel)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center);

            stack![main_view, backdrop, overlay].into()
        } else {
            main_view.into()
        }
    }
}

// ---------------------------------------------------------------------------
// Read-only preview grid
// ---------------------------------------------------------------------------

const PREVIEW_CELL: f32 = 28.0;
const PREVIEW_SIDE: f32 = PREVIEW_CELL * 16.0;

struct PreviewProgram<'a> {
    colors: &'a [Rgb; 256],
}

impl<'a> canvas::Program<Msg> for PreviewProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let sz = bounds.size();
        let mut frame = canvas::Frame::new(renderer, sz);
        let cw = sz.width / 16.0;
        let ch = sz.height / 16.0;

        for i in 0u16..256 {
            let b = i as u8;
            let x = (i % 16) as f32 * cw;
            let y = (i / 16) as f32 * ch;
            let rgb = self.colors[b as usize];

            frame.fill_rectangle(
                Point::new(x, y),
                Size::new(cw, ch),
                Color::from_rgb8(rgb.r, rgb.g, rgb.b),
            );

            let lum = 0.299 * rgb.r as f32 + 0.587 * rgb.g as f32 + 0.114 * rgb.b as f32;
            let tc = if lum > 128.0 {
                Color::BLACK
            } else {
                Color::WHITE
            };
            frame.fill_text(canvas::Text {
                content: format!("{b:02X}"),
                position: Point::new(x + cw / 2.0, y + ch / 2.0),
                color: tc,
                size: 10.0.into(),
                font: iced::Font::MONOSPACE,
                align_x: w_text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
        }

        let line_color = Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 0.25,
        };
        for j in 0..=16 {
            let x = j as f32 * cw;
            let y = j as f32 * ch;
            frame.stroke(
                &canvas::Path::line(Point::new(x, 0.0), Point::new(x, sz.height)),
                canvas::Stroke::default()
                    .with_color(line_color)
                    .with_width(0.5),
            );
            frame.stroke(
                &canvas::Path::line(Point::new(0.0, y), Point::new(sz.width, y)),
                canvas::Stroke::default()
                    .with_color(line_color)
                    .with_width(0.5),
            );
        }

        vec![frame.into_geometry()]
    }
}
