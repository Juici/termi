use std::io::{self, Write};
use std::os::fd::AsFd;
use std::time::Duration;

use uuid::Uuid;

use crate::event::{DesktopNotificationsSupport, Event, EventLoop, Filter};
use crate::terminal::{get_tty, open_dev_tty, set_raw_mode};

struct DesktopNotificationsFilter<'a> {
    identifier: &'a str,
}

impl Filter for DesktopNotificationsFilter<'_> {
    fn eval(&self, event: &Event) -> bool {
        match event {
            Event::DesktopNotifications(e) if e.identifier == self.identifier => true,
            Event::PrimaryDeviceAttributes => true,
            _ => false,
        }
    }
}

struct PrimaryDeviceAttributesFilter;

impl Filter for PrimaryDeviceAttributesFilter {
    fn eval(&self, event: &Event) -> bool {
        matches!(event, Event::PrimaryDeviceAttributes)
    }
}

pub fn query() -> io::Result<Option<DesktopNotificationsSupport>> {
    let tty = get_tty()?;
    let tty = tty.as_fd();

    let _guard = set_raw_mode(tty)?;

    // See <https://sw.kovidgoyal.net/kitty/desktop-notifications/#querying-for-support>
    //
    // ESC ] 99 ; i=<identifier> : p=? ; ESC \      Query desktop notifications support.
    // ESC [ 0 c                                    Query primary device attributes.
    //
    // Identifiers are strings consisting solely of character from the set [a-zA-Z0-9_-+.].

    let mut identifier_buffer = [0u8; uuid::fmt::Simple::LENGTH];
    let identifier: &str = Uuid::new_v4().simple().encode_lower(&mut identifier_buffer);

    let mut event_loop = EventLoop::new(tty)?;

    // Write query.
    {
        fn write_query(f: &mut impl Write, identifier: &str) -> io::Result<()> {
            // Query desktop notifications support.
            f.write_all(b"\x1b]99;i=")?;
            f.write_all(identifier.as_bytes())?;
            f.write_all(b":p=?;\x1b\\")?;

            // Query primary device attributes.
            f.write_all(b"\x1b[0c")?;

            Ok(())
        }

        let attempt = open_dev_tty()
            .map_err(io::Error::from)
            .and_then(|mut fd| write_query(&mut fd, identifier));

        if attempt.is_err() {
            let mut stdout = io::stdout().lock();
            write_query(&mut stdout, identifier)?;
            stdout.flush()?;
        }
    }

    let filter = DesktopNotificationsFilter { identifier };

    loop {
        match event_loop.poll(Some(Duration::from_secs(2)), &filter) {
            Ok(true) => match event_loop.read(&filter) {
                Ok(Event::DesktopNotifications(support)) => {
                    // Flush PrimaryDeviceAttributes from event queue.
                    let _ = event_loop.read(&PrimaryDeviceAttributesFilter);

                    return Ok(Some(support));
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
