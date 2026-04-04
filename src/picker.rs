use iced::mouse;
use iced::widget::canvas::{self, Canvas};
use iced::widget::{button, container, row, slider, text, Column, Row};
use iced::{Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

// ---------------------------------------------------------------------------
// HSV (h,s,v in [0,1]) <-> RGB8
// ---------------------------------------------------------------------------

fn rgb8_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let d = max - min;
    let v = max;
    let s = if max <= 1e-5 { 0.0 } else { d / max };
    let h_deg = if d <= 1e-5 {
        0.0
    } else if (max - rf).abs() < 1e-5 {
        let mut hh = (gf - bf) / d * 60.0;
        if gf < bf {
            hh += 360.0;
        }
        hh
    } else if (max - gf).abs() < 1e-5 {
        ((bf - rf) / d + 2.0) * 60.0
    } else {
        ((rf - gf) / d + 4.0) * 60.0
    };
    let h = (h_deg / 360.0).rem_euclid(1.0);
    (h, s, v)
}

fn hsv_to_rgb8(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = (h.fract() + 1.0).fract();
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match (h * 6.0).floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

fn hsv_to_iced_color(h: f32, s: f32, v: f32) -> Color {
    let (r, g, b) = hsv_to_rgb8(h, s, v);
    Color::from_rgb8(r, g, b)
}

// ---------------------------------------------------------------------------
// Messages & state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PickerMessage {
    HueSatFromDisc { h: f32, s: f32 },
    ValueFromBar(f32),
    RedChanged(u8),
    GreenChanged(u8),
    BlueChanged(u8),
    Ok,
    Cancel,
}

pub struct ColorPickerState {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl ColorPickerState {
    pub fn from_color(c: Color) -> Self {
        let r = (c.r * 255.0 + 0.5) as u8;
        let g = (c.g * 255.0 + 0.5) as u8;
        let b = (c.b * 255.0 + 0.5) as u8;
        let (h, s, v) = rgb8_to_hsv(r, g, b);
        Self { h, s, v }
    }

    pub fn to_color(&self) -> Color {
        let (r, g, b) = hsv_to_rgb8(self.h, self.s, self.v);
        Color::from_rgb8(r, g, b)
    }

    fn rgb8(&self) -> (u8, u8, u8) {
        hsv_to_rgb8(self.h, self.s, self.v)
    }

    pub fn update(&mut self, msg: &PickerMessage) {
        match msg {
            PickerMessage::HueSatFromDisc { h, s } => {
                self.h = *h;
                self.s = (*s).clamp(0.0, 1.0);
            }
            PickerMessage::ValueFromBar(v) => {
                self.v = (*v).clamp(0.0, 1.0);
            }
            PickerMessage::RedChanged(r) => {
                let (_, g, b) = self.rgb8();
                let (h, s, v) = rgb8_to_hsv(*r, g, b);
                self.h = h;
                self.s = s;
                self.v = v;
            }
            PickerMessage::GreenChanged(g) => {
                let (r, _, b) = self.rgb8();
                let (h, s, v) = rgb8_to_hsv(r, *g, b);
                self.h = h;
                self.s = s;
                self.v = v;
            }
            PickerMessage::BlueChanged(b) => {
                let (r, g, _) = self.rgb8();
                let (h, s, v) = rgb8_to_hsv(r, g, *b);
                self.h = h;
                self.s = s;
                self.v = v;
            }
            PickerMessage::Ok | PickerMessage::Cancel => {}
        }
    }

    pub fn view(&self) -> Element<PickerMessage> {
        let (r, g, b) = self.rgb8();
        let preview_color = self.to_color();
        let hex_label = format!("#{r:02X}{g:02X}{b:02X}");

        let preview = container(text("").size(1))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(preview_color)),
                border: Border {
                    color: Color::WHITE,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        let disc: Element<PickerMessage> = Canvas::new(SaturationDiscProgram {
            h: self.h,
            s: self.s,
        })
        .width(Length::Fixed(DISC_DIAMETER))
        .height(Length::Fixed(DISC_DIAMETER))
        .into();

        let vbar: Element<PickerMessage> = Canvas::new(ValueBarProgram {
            h: self.h,
            s: self.s,
            v: self.v,
        })
        .width(Length::Fixed(VALUE_BAR_WIDTH))
        .height(Length::Fixed(DISC_DIAMETER))
        .into();

        fn channel(label: &'static str, value: u8, on_change: fn(u8) -> PickerMessage) -> Element<'static, PickerMessage> {
            Row::new()
                .push(
                    text(label)
                        .size(13)
                        .font(iced::Font::MONOSPACE)
                        .width(Length::Fixed(16.0)),
                )
                .push(
                    slider(0..=255u8, value, on_change).width(Length::Fixed(140.0)),
                )
                .push(
                    text(format!("{value:3}"))
                        .size(13)
                        .font(iced::Font::MONOSPACE)
                        .width(Length::Fixed(32.0)),
                )
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
        }

        let sliders = Column::new()
            .push(channel("R", r, PickerMessage::RedChanged))
            .push(channel("G", g, PickerMessage::GreenChanged))
            .push(channel("B", b, PickerMessage::BlueChanged))
            .spacing(6);

        let buttons = row![
            button(text("OK").center().width(Length::Fill))
                .width(Length::Fixed(80.0))
                .on_press(PickerMessage::Ok),
            button(text("Cancel").center().width(Length::Fill))
                .width(Length::Fixed(80.0))
                .on_press(PickerMessage::Cancel),
        ]
        .spacing(8);

        let pickers_row = Row::new()
            .push(disc)
            .push(vbar)
            .spacing(10)
            .align_y(iced::Alignment::Center);

        Column::new()
            .push(
                row![preview, text(hex_label).size(14).font(iced::Font::MONOSPACE)]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
            )
            .push(
                Row::new()
                    .push(pickers_row)
                    .push(sliders)
                    .spacing(16)
                    .align_y(iced::Alignment::Center),
            )
            .push(buttons)
            .spacing(10)
            .padding(12)
            .into()
    }
}

const DISC_DIAMETER: f32 = 200.0;
const VALUE_BAR_WIDTH: f32 = 28.0;
const DISC_ANGULAR_STEPS: usize = 72;
const DISC_RADIAL_STEPS: usize = 36;

// ---------------------------------------------------------------------------
// Saturation disc: white center, hue by angle, saturation by radius (V=1 preview)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct DiscInteraction {
    dragging: bool,
}

struct SaturationDiscProgram {
    h: f32,
    s: f32,
}

impl SaturationDiscProgram {
    fn geometry(&self, size: Size) -> (Point, f32) {
        let cx = size.width * 0.5;
        let cy = size.height * 0.5;
        let radius = (size.width.min(size.height) * 0.5 - 4.0).max(1.0);
        (Point::new(cx, cy), radius)
    }

    fn pos_to_hs(&self, pos: Point, size: Size) -> Option<(f32, f32)> {
        let (c, radius) = self.geometry(size);
        let dx = pos.x - c.x;
        let dy = pos.y - c.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1e-3 {
            return Some((self.h, 0.0));
        }
        let s = (dist / radius).min(1.0);
        let h = (dy.atan2(dx) / std::f32::consts::TAU + 1.0).fract();
        Some((h, s))
    }
}

impl canvas::Program<PickerMessage> for SaturationDiscProgram {
    type State = DiscInteraction;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<PickerMessage>> {
        let size = bounds.size();
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some((h, s)) = self.pos_to_hs(pos, size) {
                        state.dragging = true;
                        return Some(
                            canvas::Action::publish(PickerMessage::HueSatFromDisc { h, s }).and_capture(),
                        );
                    }
                }
                None
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some((h, s)) = self.pos_to_hs(pos, size) {
                        return Some(
                            canvas::Action::publish(PickerMessage::HueSatFromDisc { h, s }).and_capture(),
                        );
                    }
                }
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging => {
                state.dragging = false;
                Some(canvas::Action::capture())
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
        let (c, radius) = self.geometry(sz);
        let na = DISC_ANGULAR_STEPS;
        let nr = DISC_RADIAL_STEPS;
        let tau = std::f32::consts::TAU;

        for j in 0..nr {
            let r0 = radius * j as f32 / nr as f32;
            let r1 = radius * (j + 1) as f32 / nr as f32;
            let s_mid = (r0 + r1) / (2.0 * radius);
            for i in 0..na {
                let a0 = tau * i as f32 / na as f32;
                let a1 = tau * (i + 1) as f32 / na as f32;
                let h_mid = (i as f32 + 0.5) / na as f32;
                let color = hsv_to_iced_color(h_mid, s_mid, 1.0);
                let path = if r0 <= 0.01 {
                    let p1 = Point::new(c.x + r1 * a0.cos(), c.y + r1 * a0.sin());
                    let p2 = Point::new(c.x + r1 * a1.cos(), c.y + r1 * a1.sin());
                    canvas::Path::new(|b| {
                        b.move_to(c);
                        b.line_to(p1);
                        b.line_to(p2);
                        b.close();
                    })
                } else {
                    let p00 = Point::new(c.x + r0 * a0.cos(), c.y + r0 * a0.sin());
                    let p01 = Point::new(c.x + r0 * a1.cos(), c.y + r0 * a1.sin());
                    let p10 = Point::new(c.x + r1 * a0.cos(), c.y + r1 * a0.sin());
                    let p11 = Point::new(c.x + r1 * a1.cos(), c.y + r1 * a1.sin());
                    canvas::Path::new(|b| {
                        b.move_to(p00);
                        b.line_to(p10);
                        b.line_to(p11);
                        b.line_to(p01);
                        b.close();
                    })
                };
                frame.fill(&path, color);
            }
        }

        let stroke_col = Color {
            r: 0.4,
            g: 0.4,
            b: 0.4,
            a: 0.9,
        };
        frame.stroke(
            &canvas::Path::circle(c, radius),
            canvas::Stroke::default().with_color(stroke_col).with_width(1.0),
        );

        let sel_a = self.h * tau;
        let sel_r = self.s * radius;
        let sx = c.x + sel_r * sel_a.cos();
        let sy = c.y + sel_r * sel_a.sin();
        let sel_pt = Point::new(sx, sy);
        frame.stroke(
            &canvas::Path::circle(sel_pt, 5.0),
            canvas::Stroke::default().with_color(Color::WHITE).with_width(2.0),
        );
        frame.stroke(
            &canvas::Path::circle(sel_pt, 5.0),
            canvas::Stroke::default().with_color(Color::BLACK).with_width(1.0),
        );

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

// ---------------------------------------------------------------------------
// Value bar: V from 1 (top) to 0 (bottom) at current H,S
// ---------------------------------------------------------------------------

#[derive(Default)]
struct BarInteraction {
    dragging: bool,
}

struct ValueBarProgram {
    h: f32,
    s: f32,
    v: f32,
}

impl canvas::Program<PickerMessage> for ValueBarProgram {
    type State = BarInteraction;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<PickerMessage>> {
        let h = bounds.height.max(1.0);
        let pick_v = |y: f32| (1.0 - (y / h).clamp(0.0, 1.0)).clamp(0.0, 1.0);

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.dragging = true;
                    let v = pick_v(pos.y);
                    return Some(canvas::Action::publish(PickerMessage::ValueFromBar(v)).and_capture());
                }
                None
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let v = pick_v(pos.y);
                    return Some(canvas::Action::publish(PickerMessage::ValueFromBar(v)).and_capture());
                }
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging => {
                state.dragging = false;
                Some(canvas::Action::capture())
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
        let w = sz.width;
        let h = sz.height;
        let steps = 32;
        let strip_h = h / steps as f32;
        for i in 0..steps {
            let v0 = 1.0 - i as f32 / steps as f32;
            let v1 = 1.0 - (i + 1) as f32 / steps as f32;
            let v_mid = (v0 + v1) * 0.5;
            let color = hsv_to_iced_color(self.h, self.s, v_mid);
            let y0 = i as f32 * strip_h;
            frame.fill_rectangle(Point::new(0.0, y0), Size::new(w, strip_h), color);
        }

        let vy = (1.0 - self.v) * h;
        frame.stroke(
            &canvas::Path::line(Point::new(0.0, vy), Point::new(w, vy)),
            canvas::Stroke::default().with_color(Color::WHITE).with_width(2.0),
        );
        frame.stroke_rectangle(
            Point::new(0.0, 0.0),
            Size::new(w, h),
            canvas::Stroke::default()
                .with_color(Color {
                    r: 0.4,
                    g: 0.4,
                    b: 0.4,
                    a: 0.9,
                })
                .with_width(1.0),
        );

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
