use hex_color::Rgb;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::text as w_text;
use iced::{alignment, Color, Point, Rectangle, Renderer, Size, Theme};

pub const CELL_SIZE: f32 = 36.0;
pub const GRID_SIDE: f32 = CELL_SIZE * 16.0;

#[derive(Debug, Clone)]
pub enum GridMessage {
    SelectionChanged { start: u8, end: u8 },
    DragEnded,
}

pub struct GridProgram<'a> {
    pub colors: &'a [Rgb; 256],
    pub selection: Option<(u8, u8)>,
}

#[derive(Default)]
pub struct GridInteraction {
    anchor: Option<u8>,
    dragging: bool,
}

fn hit_test(pos: Point, size: Size) -> Option<u8> {
    let cw = size.width / 16.0;
    let ch = size.height / 16.0;
    let col = (pos.x / cw) as usize;
    let row = (pos.y / ch) as usize;
    if col < 16 && row < 16 {
        Some((row * 16 + col) as u8)
    } else {
        None
    }
}

impl<'a> canvas::Program<GridMessage> for GridProgram<'a> {
    type State = GridInteraction;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<GridMessage>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some(b) = hit_test(pos, bounds.size()) {
                        state.anchor = Some(b);
                        state.dragging = true;
                        return Some(
                            canvas::Action::publish(GridMessage::SelectionChanged {
                                start: b,
                                end: b,
                            })
                            .and_capture(),
                        );
                    }
                }
                None
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some(b) = hit_test(pos, bounds.size()) {
                        if let Some(a) = state.anchor {
                            return Some(
                                canvas::Action::publish(GridMessage::SelectionChanged {
                                    start: a.min(b),
                                    end: a.max(b),
                                })
                                .and_capture(),
                            );
                        }
                    }
                }
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                Some(canvas::Action::publish(GridMessage::DragEnded).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
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

            if let Some((lo, hi)) = self.selection {
                if b >= lo && b <= hi {
                    frame.fill_rectangle(
                        Point::new(x, y),
                        Size::new(cw, ch),
                        Color {
                            r: 0.3,
                            g: 0.55,
                            b: 1.0,
                            a: 0.25,
                        },
                    );
                    frame.stroke(
                        &canvas::Path::rectangle(
                            Point::new(x + 1.0, y + 1.0),
                            Size::new(cw - 2.0, ch - 2.0),
                        ),
                        canvas::Stroke::default()
                            .with_color(Color {
                                r: 0.4,
                                g: 0.65,
                                b: 1.0,
                                a: 0.9,
                            })
                            .with_width(1.5),
                    );
                }
            }

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
                size: 11.0.into(),
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
            a: 0.3,
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

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Crosshair
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
