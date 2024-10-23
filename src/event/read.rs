use std::collections::VecDeque;
use std::io;
use std::mem::MaybeUninit;
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags};
use rustix::io::Errno;

use crate::event::{Event, Parser};
use crate::fd::BorrowedFd;

pub trait Filter {
    fn eval(&self, event: &Event) -> bool;
}

impl<T> Filter for T
where
    T: Fn(&Event) -> bool,
{
    fn eval(&self, event: &Event) -> bool {
        self(event)
    }
}

const TTY_BUFFER_LEN: usize = 1024;

pub struct EventLoop<'fd> {
    tty_fd: BorrowedFd<'fd>,
    tty_buffer: [MaybeUninit<u8>; TTY_BUFFER_LEN],
    parser: Parser,
    events: VecDeque<Event>,
    skipped_events: Vec<Event>,
}

impl<'fd> EventLoop<'fd> {
    pub fn new(tty_fd: BorrowedFd<'fd>) -> io::Result<Self> {
        rustix::termios::tcflush(tty_fd, rustix::termios::QueueSelector::IFlush)?;

        Ok(Self {
            tty_fd,
            tty_buffer: [MaybeUninit::uninit(); TTY_BUFFER_LEN],
            parser: Parser::new(),
            events: VecDeque::with_capacity(32),
            skipped_events: Vec::with_capacity(32),
        })
    }

    fn poll_internal(&self, timeout: Option<Duration>) -> io::Result<bool> {
        loop {
            // A bug in kernels < 2.6.37 makes timeouts larger than LONG_MAX / CONFIG_HZ
            // (approx. 30 minutes with CONFIG_HZ=1200) effectively infinite on 32 bits
            // architectures. The magic number is the same constant used by libuv.
            #[cfg(target_pointer_width = "32")]
            const MAX_SAFE_TIMEOUT: u128 = 1789569;
            #[cfg(not(target_pointer_width = "32"))]
            const MAX_SAFE_TIMEOUT: u128 = i32::MAX as u128;

            let timeout = timeout
                .map(|d| {
                    // `Duration::as_millis` truncates, so round up. This avoids turning
                    // sub-millisecond timeouts into a zero timeout, unless the caller
                    // explicitly requests that by specifying a zero timeout.
                    d.checked_add(Duration::from_nanos(999_999))
                        .unwrap_or(d)
                        .as_millis()
                        .min(MAX_SAFE_TIMEOUT) as i32
                })
                .unwrap_or(-1);

            let mut fds = [PollFd::from_borrowed_fd(self.tty_fd, PollFlags::IN)];

            match rustix::event::poll(&mut fds, timeout) {
                Ok(num_events) => break Ok(num_events != 0),
                Err(err) if err == Errno::AGAIN => continue,
                Err(err) => return Err(err.into()),
            }
        }
    }

    fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<Event>> {
        if let Some(event) = self.parser.next() {
            return Ok(Some(event));
        }

        let timeout = PollTimeout::new(timeout);

        let mut leftover = timeout.leftover();
        loop {
            match self.poll_internal(leftover) {
                Ok(true) => 'read: loop {
                    match rustix::io::read_uninit(self.tty_fd, &mut self.tty_buffer) {
                        Ok((buf, _)) => {
                            if !buf.is_empty() {
                                self.parser.advance(buf);
                            }
                        }
                        Err(err) => match err.kind() {
                            io::ErrorKind::WouldBlock => break 'read,
                            io::ErrorKind::Interrupted => continue 'read,
                            _ => {}
                        },
                    }

                    if let Some(event) = self.parser.next() {
                        return Ok(Some(event));
                    }
                },
                Ok(false) => return Ok(None),
                Err(err) => {
                    if err.kind() != io::ErrorKind::Interrupted {
                        return Err(err);
                    }
                }
            }

            leftover = match timeout.leftover() {
                Some(leftover) if leftover.is_zero() => return Ok(None),
                leftover => leftover,
            };
        }
    }

    pub fn poll<F>(&mut self, timeout: Option<Duration>, filter: &F) -> io::Result<bool>
    where
        F: Filter,
    {
        for event in &self.events {
            if filter.eval(event) {
                return Ok(true);
            }
        }

        let timeout = PollTimeout::new(timeout);

        let mut leftover = timeout.leftover();
        loop {
            let event = match self.try_read(leftover) {
                Ok(None) => None,
                Ok(Some(event)) => {
                    if filter.eval(&event) {
                        Some(event)
                    } else {
                        self.skipped_events.push(event);
                        None
                    }
                }
                Err(err) => {
                    if err.kind() == io::ErrorKind::Interrupted {
                        return Ok(false);
                    }
                    return Err(err);
                }
            };

            leftover = timeout.leftover();

            if leftover.map_or(false, |t| t.is_zero()) || event.is_some() {
                self.events.extend(self.skipped_events.drain(..));

                if let Some(event) = event {
                    self.events.push_front(event);
                    return Ok(true);
                }

                return Ok(false);
            }
        }
    }

    pub fn read<F>(&mut self, filter: &F) -> io::Result<Event>
    where
        F: Filter,
    {
        loop {
            while let Some(event) = self.events.pop_front() {
                if filter.eval(&event) {
                    self.events.extend(self.skipped_events.drain(..));

                    return Ok(event);
                } else {
                    self.skipped_events.push(event);
                }
            }

            let _ = self.poll(None, filter)?;
        }
    }
}

enum PollTimeout {
    Timeout { timeout: Duration, start: Instant },
    None,
    Infinite,
}

impl PollTimeout {
    fn new(timeout: Option<Duration>) -> Self {
        match timeout {
            Some(timeout) if timeout.is_zero() => Self::None,
            Some(timeout) => Self::Timeout { timeout, start: Instant::now() },
            None => Self::Infinite,
        }
    }

    fn leftover(&self) -> Option<Duration> {
        match self {
            Self::Timeout { timeout, start } => {
                let elapsed = start.elapsed();
                let timeout = *timeout;

                Some(if elapsed >= timeout { Duration::ZERO } else { timeout - elapsed })
            }
            Self::None => Some(Duration::ZERO),
            Self::Infinite => None,
        }
    }
}
