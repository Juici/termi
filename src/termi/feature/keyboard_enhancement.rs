use std::io::{self, Write};
use std::os::fd::AsFd;
use std::time::Duration;

use crate::event::{Event, EventLoop, Filter, KeyboardEnhancementFlags};
use crate::terminal::{get_tty, open_dev_tty, set_raw_mode};

struct KeyboardEnchancementFlagsFilter;

impl Filter for KeyboardEnchancementFlagsFilter {
    fn eval(&self, event: &Event) -> bool {
        matches!(event, Event::KeyboardEnhancmentFlags(_) | Event::PrimaryDeviceAttributes)
    }
}

struct PrimaryDeviceAttributesFilter;

impl Filter for PrimaryDeviceAttributesFilter {
    fn eval(&self, event: &Event) -> bool {
        matches!(event, Event::PrimaryDeviceAttributes)
    }
}

pub fn query() -> io::Result<Option<KeyboardEnhancementFlags>> {
    let tty = get_tty()?;
    let tty = tty.as_fd();

    let _guard = set_raw_mode(tty)?;

    // This is the recommended method for testing support for the keyboard enhancement protocol.
    // We send a query for the flags supported by the terminal and then the primary device attributes
    // query. If we receive the primary device attributes response but not the keyboard enhancement
    // flags, none of the flags are supported.
    //
    // See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#detection-of-support-for-this-protocol>
    //
    // ESC [ ? u        Query progressive keyboard enhancement flags (kitty protocol).
    // ESC [ 0 c        Query primary device attributes.
    const QUERY: &[u8] = b"\x1b[?u\x1b[0c";

    let mut event_loop = EventLoop::new(tty)?;

    // Write query.
    {
        let attempt =
            open_dev_tty().map_err(io::Error::from).and_then(|mut fd| fd.write_all(QUERY));

        if attempt.is_err() {
            let mut stdout = io::stdout().lock();
            stdout.write_all(QUERY)?;
            stdout.flush()?;
        }
    }

    loop {
        match event_loop.poll(Some(Duration::from_secs(2)), &KeyboardEnchancementFlagsFilter) {
            Ok(true) => match event_loop.read(&KeyboardEnchancementFlagsFilter) {
                Ok(Event::KeyboardEnhancmentFlags(flags)) => {
                    // Flush PrimaryDeviceAttributes from event queue.
                    let _ = event_loop.read(&PrimaryDeviceAttributesFilter);

                    return Ok(Some(flags));
                }
                _ => return Ok(None),
            },
            Ok(false) => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Keyboard enhancement status could not be read within a normal duration",
                ))
            }
            Err(_) => {}
        }
    }
}
