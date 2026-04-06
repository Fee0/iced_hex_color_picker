mod editor;
mod style;

pub use editor::{ColorMapEditor, Event, Message, PresetKind};
pub use iced_color_picker::PickerMessage;
pub use style::{ColorMapEditorStyle, DEFAULT_GRID_CELL_SIZE, GridDrawStyle, MapColorTarget};
