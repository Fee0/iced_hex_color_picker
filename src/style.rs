//! Visual configuration for [`crate::editor::ColorMapEditor`].
use iced::Color;

/// Default edge length of one grid cell in logical pixels (used by [`GridDrawStyle::default`]).
pub const DEFAULT_GRID_CELL_SIZE: f32 = 36.0;

/// Canvas styling for the 16×16 byte grid (`Copy` for cheap passes into [`crate::grid::GridProgram`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridDrawStyle {
    /// Uniform fill behind each cell (hex labels use the map color).
    pub cell_background: Color,
    /// Semi-transparent overlay on the selected index range.
    pub selection_overlay: Color,
    /// Stroke color for the selection union outline.
    pub selection_outline: Color,
    pub selection_outline_width: f32,
    pub grid_line_color: Color,
    /// Edge length of one cell in logical pixels; total grid side is `16 * cell_size`.
    pub cell_size: f32,
}

impl GridDrawStyle {
    #[inline]
    pub fn grid_side(self) -> f32 {
        self.cell_size * 16.0
    }
}

impl Default for GridDrawStyle {
    fn default() -> Self {
        Self {
            cell_background: Color::from_rgb8(0x2A, 0x2A, 0x2A),
            selection_overlay: Color {
                r: 0.3,
                g: 0.55,
                b: 1.0,
                a: 0.28,
            },
            selection_outline: Color {
                r: 0.4,
                g: 0.65,
                b: 1.0,
                a: 0.9,
            },
            selection_outline_width: 1.5,
            grid_line_color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 0.3,
            },
            cell_size: DEFAULT_GRID_CELL_SIZE,
        }
    }
}

/// Editor widget chrome and grid canvas appearance.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorMapEditorStyle {
    pub grid: GridDrawStyle,
    pub grid_border_color: Color,
    pub grid_border_width: f32,
    pub grid_border_radius: f32,
    pub show_presets: bool,
}

impl Default for ColorMapEditorStyle {
    fn default() -> Self {
        Self {
            grid: GridDrawStyle::default(),
            grid_border_color: Color {
                r: 0.55,
                g: 0.55,
                b: 0.58,
                a: 0.85,
            },
            grid_border_width: 1.0,
            grid_border_radius: 4.0,
            show_presets: true,
        }
    }
}

