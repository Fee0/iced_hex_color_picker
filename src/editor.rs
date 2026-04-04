use crate::grid::{GridMessage, GridProgram, GRID_SIDE};
use crate::picker::{ColorPickerState, PickerMessage};
use hex_color::presets::{ascii_classes, nibble_groups, AsciiClassColors, NibbleGroupColors};
use hex_color::{ColorMap, Rgb};
use iced::widget::canvas::Canvas;
use iced::widget::{button, container, text};
use iced::{Color, Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    Grid(GridMessage),
    OpenPicker,
    Picker(PickerMessage),
    LoadAsciiClasses,
    LoadNibbleGroups,
    FillAll,
    Accept,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum Event {
    Accepted(ColorMap),
    Cancelled,
}

pub struct ColorMapEditor {
    draft: [Rgb; 256],
    baseline: [Rgb; 256],
    selection: Option<(u8, u8)>,
    picker_open: bool,
    picker_state: ColorPickerState,
}

fn rgb_to_iced(rgb: Rgb) -> Color {
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

fn iced_to_rgb(c: Color) -> Rgb {
    Rgb::new(
        (c.r * 255.0 + 0.5) as u8,
        (c.g * 255.0 + 0.5) as u8,
        (c.b * 255.0 + 0.5) as u8,
    )
}

impl ColorMapEditor {
    pub fn new(initial: &ColorMap) -> Self {
        let table = *initial.as_table();
        Self {
            draft: table,
            baseline: table,
            selection: None,
            picker_open: false,
            picker_state: ColorPickerState::from_color(rgb_to_iced(table[0])),
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Grid(GridMessage::SelectionChanged { start, end }) => {
                self.selection = Some((start, end));
                let c = rgb_to_iced(self.draft[start as usize]);
                self.picker_state = ColorPickerState::from_color(c);
            }
            Message::Grid(GridMessage::DragEnded) => {
                self.picker_open = true;
            }
            Message::OpenPicker => {
                self.picker_open = true;
            }
            Message::Picker(ref inner) => {
                self.picker_state.update(inner);
                match inner {
                    PickerMessage::Ok => {
                        self.picker_open = false;
                        let color = self.picker_state.to_color();
                        if let Some((start, end)) = self.selection {
                            let rgb = iced_to_rgb(color);
                            for i in start..=end {
                                self.draft[i as usize] = rgb;
                            }
                        }
                    }
                    PickerMessage::Cancel => {
                        self.picker_open = false;
                    }
                    _ => {}
                }
            }
            Message::LoadAsciiClasses => {
                self.picker_open = false;
                self.selection = None;
                let map = ascii_classes(AsciiClassColors {
                    null: Rgb::from_hex(0x404040),
                    printable: Rgb::from_hex(0x00CC00),
                    whitespace: Rgb::from_hex(0xCCCC00),
                    control: Rgb::from_hex(0xCC0000),
                    non_ascii: Rgb::from_hex(0x0066CC),
                });
                self.draft = *map.as_table();
            }
            Message::LoadNibbleGroups => {
                self.picker_open = false;
                self.selection = None;
                let map = nibble_groups(NibbleGroupColors {
                    zero: Rgb::from_hex(0x222222),
                    leading_nibbles: [
                        Rgb::from_hex(0x1A0A2E),
                        Rgb::from_hex(0x2D1B69),
                        Rgb::from_hex(0x16213E),
                        Rgb::from_hex(0x0F3460),
                        Rgb::from_hex(0x1A4045),
                        Rgb::from_hex(0x1B5E20),
                        Rgb::from_hex(0x33691E),
                        Rgb::from_hex(0x827717),
                        Rgb::from_hex(0xF57F17),
                        Rgb::from_hex(0xFF6F00),
                        Rgb::from_hex(0xE65100),
                        Rgb::from_hex(0xBF360C),
                        Rgb::from_hex(0xB71C1C),
                        Rgb::from_hex(0x880E4F),
                        Rgb::from_hex(0x4A148C),
                        Rgb::from_hex(0x311B92),
                    ],
                    ff: Rgb::from_hex(0xFFFFFF),
                });
                self.draft = *map.as_table();
            }
            Message::FillAll => {
                let rgb = iced_to_rgb(self.picker_state.to_color());
                self.draft = [rgb; 256];
            }
            Message::Accept => {
                return Some(Event::Accepted(ColorMap::new(self.draft)));
            }
            Message::Cancel => {
                self.draft = self.baseline;
                return Some(Event::Cancelled);
            }
        }
        None
    }

    pub fn view(&self) -> Element<Message> {
        let toolbar = iced::widget::Row::new()
            .push(button(text("ASCII Classes")).on_press(Message::LoadAsciiClasses))
            .push(button(text("Nibble Groups")).on_press(Message::LoadNibbleGroups))
            .push(button(text("Fill All")).on_press(Message::FillAll))
            .spacing(8);

        let program = GridProgram {
            colors: &self.draft,
            selection: self.selection,
        };
        let grid: Element<GridMessage> = Canvas::new(program)
            .width(Length::Fixed(GRID_SIDE))
            .height(Length::Fixed(GRID_SIDE))
            .into();
        let grid = grid.map(Message::Grid);

        let sel_label = match self.selection {
            Some((s, e)) if s == e => format!("Selected: 0x{s:02X}"),
            Some((s, e)) => format!("Selected: 0x{s:02X}..=0x{e:02X}"),
            None => "Click a cell to select".into(),
        };

        let mut col = iced::widget::Column::new()
            .push(toolbar)
            .push(grid)
            .push(text(sel_label).size(14));

        if self.picker_open {
            col = col.push(self.picker_state.view().map(Message::Picker));
        } else {
            let pick_btn = if self.selection.is_some() {
                button(text("Pick Color")).on_press(Message::OpenPicker)
            } else {
                button(text("Pick Color"))
            };
            col = col.push(pick_btn);
        }

        let actions = iced::widget::Row::new()
            .push(button(text("Accept")).on_press(Message::Accept))
            .push(button(text("Cancel")).on_press(Message::Cancel))
            .spacing(8);

        col = col.push(actions);

        container(col.spacing(12).padding(16))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
