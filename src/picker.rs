use iced::widget::{button, container, row, slider, text, Column, Row};
use iced::{Background, Border, Color, Element, Length, Theme};

#[derive(Debug, Clone)]
pub enum PickerMessage {
    RedChanged(u8),
    GreenChanged(u8),
    BlueChanged(u8),
    Ok,
    Cancel,
}

pub struct ColorPickerState {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorPickerState {
    pub fn from_color(c: Color) -> Self {
        Self {
            r: (c.r * 255.0 + 0.5) as u8,
            g: (c.g * 255.0 + 0.5) as u8,
            b: (c.b * 255.0 + 0.5) as u8,
        }
    }

    pub fn to_color(&self) -> Color {
        Color::from_rgb8(self.r, self.g, self.b)
    }

    pub fn update(&mut self, msg: &PickerMessage) {
        match msg {
            PickerMessage::RedChanged(v) => self.r = *v,
            PickerMessage::GreenChanged(v) => self.g = *v,
            PickerMessage::BlueChanged(v) => self.b = *v,
            PickerMessage::Ok | PickerMessage::Cancel => {}
        }
    }

    pub fn view(&self) -> Element<PickerMessage> {
        let preview_color = self.to_color();
        let hex_label = format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b);

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

        fn channel(label: &'static str, value: u8, on_change: fn(u8) -> PickerMessage) -> Element<'static, PickerMessage> {
            Row::new()
                .push(
                    text(label)
                        .size(13)
                        .font(iced::Font::MONOSPACE)
                        .width(Length::Fixed(16.0)),
                )
                .push(
                    slider(0..=255u8, value, on_change)
                        .width(Length::Fixed(180.0)),
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
            .push(channel("R", self.r, PickerMessage::RedChanged))
            .push(channel("G", self.g, PickerMessage::GreenChanged))
            .push(channel("B", self.b, PickerMessage::BlueChanged))
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

        Column::new()
            .push(
                row![preview, text(hex_label).size(14).font(iced::Font::MONOSPACE)]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
            )
            .push(sliders)
            .push(buttons)
            .spacing(10)
            .padding(12)
            .into()
    }
}
