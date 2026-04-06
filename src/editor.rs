use crate::grid::{GridMessage, GridProgram};
use crate::picker::{ColorPickerState, PICKER_PANEL_WIDTH};

pub use crate::picker::PickerMessage;
pub use crate::style::{ColorMapEditorStyle, GridDrawStyle, MapColorTarget};
use hex_color::presets::{ascii_classes, nibble_groups, AsciiClassColors, NibbleGroupColors};
use hex_color::{ColorMap, Rgb};
use iced::widget::canvas::Canvas;
use iced::widget::{button, container, pick_list, text, Column, Row, Space};
use iced::{Border, Color, Element, Length, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    AsciiClasses,
    NibbleGroups,
}

impl std::fmt::Display for PresetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PresetKind::AsciiClasses => "ASCII classes",
            PresetKind::NibbleGroups => "Nibble groups",
        })
    }
}

const PRESET_OPTIONS: [PresetKind; 2] = [PresetKind::AsciiClasses, PresetKind::NibbleGroups];

#[derive(Debug, Clone)]
pub enum Message {
    Grid(GridMessage),
    Picker(PickerMessage),
    PresetSelected(PresetKind),
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
    picker_state: ColorPickerState,
    active_preset: Option<PresetKind>,
    style: ColorMapEditorStyle,
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

fn apply_picker_to_selection(draft: &mut [Rgb; 256], selection: Option<(u8, u8)>, picker: &ColorPickerState) {
    if let Some((start, end)) = selection {
        let rgb = iced_to_rgb(picker.to_color());
        for i in start..=end {
            draft[i as usize] = rgb;
        }
    }
}

impl ColorMapEditor {
    pub fn new(initial: &ColorMap) -> Self {
        Self::new_with_style(initial, ColorMapEditorStyle::default())
    }

    pub fn new_with_style(initial: &ColorMap, style: ColorMapEditorStyle) -> Self {
        let table = *initial.as_table();
        Self {
            draft: table,
            baseline: table,
            selection: Some((0, 0)),
            picker_state: ColorPickerState::from_color(rgb_to_iced(table[0])),
            active_preset: None,
            style,
        }
    }

    pub fn style(&self) -> &ColorMapEditorStyle {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut ColorMapEditorStyle {
        &mut self.style
    }

    pub fn set_style(&mut self, style: ColorMapEditorStyle) {
        self.style = style;
    }

    pub fn set_grid_draw_style(&mut self, grid: GridDrawStyle) {
        self.style.grid = grid;
    }

    pub fn set_grid_cell_background(&mut self, c: Color) {
        self.style.grid.cell_background = c;
    }

    pub fn set_selection_overlay(&mut self, c: Color) {
        self.style.grid.selection_overlay = c;
    }

    pub fn set_selection_outline(&mut self, c: Color) {
        self.style.grid.selection_outline = c;
    }

    pub fn set_selection_outline_width(&mut self, w: f32) {
        self.style.grid.selection_outline_width = w;
    }

    pub fn set_grid_line_color(&mut self, c: Color) {
        self.style.grid.grid_line_color = c;
    }

    pub fn set_grid_cell_size(&mut self, s: f32) {
        self.style.grid.cell_size = s;
    }

    pub fn set_map_color_target(&mut self, target: MapColorTarget) {
        self.style.grid.map_color_target = target;
    }

    pub fn set_grid_border_color(&mut self, c: Color) {
        self.style.grid_border_color = c;
    }

    pub fn set_grid_border_width(&mut self, w: f32) {
        self.style.grid_border_width = w;
    }

    pub fn set_grid_border_radius(&mut self, r: f32) {
        self.style.grid_border_radius = r;
    }

    pub fn set_show_presets(&mut self, show: bool) {
        self.style.show_presets = show;
    }

    pub fn picker_hex_string(&self) -> String {
        let c = self.picker_state.to_color();
        let r = (c.r * 255.0 + 0.5) as u8;
        let g = (c.g * 255.0 + 0.5) as u8;
        let b = (c.b * 255.0 + 0.5) as u8;
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Grid(GridMessage::SelectionChanged { start, end }) => {
                self.selection = Some((start, end));
                let c = rgb_to_iced(self.draft[start as usize]);
                self.picker_state = ColorPickerState::from_color(c);
            }
            Message::Grid(GridMessage::DragEnded) => {}
            Message::Picker(ref inner) => {
                if matches!(inner, PickerMessage::CopyHex) {
                    return None;
                }
                self.picker_state.update(inner);
                self.active_preset = None;
                apply_picker_to_selection(&mut self.draft, self.selection, &self.picker_state);
            }
            Message::PresetSelected(kind) => {
                self.active_preset = Some(kind);
                self.selection = None;
                match kind {
                    PresetKind::AsciiClasses => {
                        let map = ascii_classes(AsciiClassColors {
                            null: Rgb::from_hex(0x404040),
                            printable: Rgb::from_hex(0x00CC00),
                            whitespace: Rgb::from_hex(0xCCCC00),
                            control: Rgb::from_hex(0xCC0000),
                            non_ascii: Rgb::from_hex(0x0066CC),
                        });
                        self.draft = *map.as_table();
                    }
                    PresetKind::NibbleGroups => {
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
                }
                self.picker_state = ColorPickerState::from_color(rgb_to_iced(self.draft[0]));
            }
            Message::Accept => {
                return Some(Event::Accepted(ColorMap::new(self.draft)));
            }
            Message::Cancel => {
                self.draft = self.baseline;
                self.active_preset = None;
                return Some(Event::Cancelled);
            }
        }
        None
    }

    pub fn view(&self) -> Element<Message> {
        let grid_side = self.style.grid.grid_side();

        let program = GridProgram {
            colors: &self.draft,
            selection: self.selection,
            draw_style: self.style.grid,
        };
        let grid: Element<GridMessage> = Canvas::new(program)
            .width(Length::Fixed(grid_side))
            .height(Length::Fixed(grid_side))
            .into();
        let grid = grid.map(Message::Grid);

        let bc = self.style.grid_border_color;
        let bw = self.style.grid_border_width;
        let br = self.style.grid_border_radius;
        let grid_bordered = container(grid).style(move |_theme: &Theme| container::Style {
            border: Border {
                color: bc,
                width: bw,
                radius: br.into(),
            },
            ..Default::default()
        });

        let left = container(grid_bordered).width(Length::Shrink);

        let actions = Row::new()
            .push(Space::new().width(Length::Fill))
            .push(button(text("Cancel")).on_press(Message::Cancel))
            .push(button(text("Accept")).on_press(Message::Accept))
            .spacing(8)
            .width(Length::Fill);

        let mut right_col = Column::new().spacing(12).width(Length::Fixed(PICKER_PANEL_WIDTH));
        if self.style.show_presets {
            let preset_dd: Element<Message> = pick_list(
                PRESET_OPTIONS,
                self.active_preset,
                Message::PresetSelected,
            )
            .placeholder("Preset…")
            .width(Length::Fill)
            .into();
            right_col = right_col.push(preset_dd);
        }
        right_col = right_col
            .push(self.picker_state.view().map(Message::Picker))
            .push(Space::new().height(Length::Fill))
            .push(actions);

        let right = container(right_col)
            .width(Length::Fixed(PICKER_PANEL_WIDTH))
            .height(Length::Fill);

        let body = Row::new()
            .push(left)
            .push(right)
            .spacing(16);

        container(body.padding(16))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into()
    }
}