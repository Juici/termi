mod parser;
mod read;

use bitflags::bitflags;

pub use self::parser::*;
pub use self::read::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    // Key(KeyEvent),
    // Mouse(MouseEvent),
    // Paste(String),
    KeyboardEnhancmentFlags(KeyboardEnhancementFlags),
    PrimaryDeviceAttributes,
    DesktopNotifications(DesktopNotificationsSupport),
}

bitflags! {
    /// Represents special flags that tell compatible terminals to add extra information to keyboard events.
    ///
    /// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement> for more information.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct KeyboardEnhancementFlags: u8 {
        const DISAMBIGUATE_ESCAPE_CODES = 1 << 0;
        const REPORT_EVENT_TYPES = 1 << 1;
        const REPORT_ALTERNATE_KEYS = 1 << 2;
        const REPORT_ALL_KEYS_AS_ESCAPE_CODES = 1 << 3;
        const REPORT_ASSOCIATED_TEXT = 1 << 4;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopNotificationsSupport {
    pub identifier: String,
    // TODO: Support key-values.
}
