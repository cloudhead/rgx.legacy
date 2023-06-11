use std::time;

use crate::math::{Point, Size};
use crate::platform;

pub use platform::InputState;

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    MouseDown(platform::MouseButton),
    MouseUp(platform::MouseButton),
    MouseScroll(platform::LogicalDelta),
    MouseMove(Point),
    Resized(Size),
    MouseEnter(Point),
    MouseExit,
    Focus(bool),
    KeyDown {
        key: platform::Key,
        modifiers: platform::ModifiersState,
        repeat: bool,
    },
    KeyUp {
        key: platform::Key,
        modifiers: platform::ModifiersState,
    },
    CharacterReceived(char, platform::ModifiersState),
    Paste(Option<String>),
    Tick(time::Duration),
    Frame,
}
