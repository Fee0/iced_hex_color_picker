mod editor;
mod style;

pub use editor::{ColorMapEditor, Event, Message, PickerMessage, PresetKind};
pub use style::{ColorMapEditorStyle, DEFAULT_GRID_CELL_SIZE, GridDrawStyle, MapColorTarget};
