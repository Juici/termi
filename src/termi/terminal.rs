use std::io;

use rustix::fs::{Mode, OFlags};
use rustix::termios::Termios;

use crate::fd::{AsFd, BorrowedFd, FileDesc, OwnedFd};

pub fn open_dev_tty() -> io::Result<OwnedFd> {
    let fd = rustix::fs::open(
        c"/dev/tty",
        OFlags::CLOEXEC | OFlags::RDWR,
        Mode::RUSR | Mode::WUSR | Mode::RGRP | Mode::WGRP | Mode::ROTH | Mode::WOTH,
    )?;
    Ok(OwnedFd::from(fd))
}

pub fn get_tty() -> io::Result<FileDesc<'static>> {
    let stdin = rustix::stdio::stdin();
    let fd = if rustix::termios::isatty(stdin) {
        FileDesc::Borrowed(stdin)
    } else {
        FileDesc::Owned(open_dev_tty()?)
    };
    Ok(fd)
}

pub fn get_terminal_attr(fd: impl AsFd) -> io::Result<Termios> {
    Ok(rustix::termios::tcgetattr(fd)?)
}

pub fn set_terminal_attr(fd: impl AsFd, termios: &Termios) -> io::Result<()> {
    rustix::termios::tcsetattr(fd, rustix::termios::OptionalActions::Now, termios)?;
    Ok(())
}

pub struct RawModeGuard<'fd> {
    fd: BorrowedFd<'fd>,
    original_ios: Termios,
}

impl Drop for RawModeGuard<'_> {
    fn drop(&mut self) {
        let _ = set_terminal_attr(self.fd, &self.original_ios);
    }
}

pub fn set_raw_mode(fd: BorrowedFd) -> io::Result<RawModeGuard> {
    let original_ios = get_terminal_attr(fd)?;

    let mut ios = original_ios.clone();
    ios.make_raw();
    set_terminal_attr(fd, &ios)?;

    Ok(RawModeGuard { fd, original_ios })
}
