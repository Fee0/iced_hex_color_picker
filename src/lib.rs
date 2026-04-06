mod editor;
mod style;

pub use editor::{ColorMapEditor, Event, Message, PickerMessage, PresetKind};
pub use style::{ColorMapEditorStyle, GridDrawStyle, MapColorTarget, DEFAULT_GRID_CELL_SIZE};
