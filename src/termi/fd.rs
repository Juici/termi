use core::mem::ManuallyDrop;
use std::io::{self, Write};

use rustix::fd;

pub use rustix::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};

#[repr(transparent)]
pub struct OwnedFd(ManuallyDrop<fd::OwnedFd>);

impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        // Manually drop to take advantage of rustix potentially using raw syscalls.
        unsafe { rustix::io::close(self.0.as_raw_fd()) };
    }
}

impl From<fd::OwnedFd> for OwnedFd {
    #[inline]
    fn from(value: fd::OwnedFd) -> Self {
        Self(ManuallyDrop::new(value))
    }
}

impl AsFd for OwnedFd {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for OwnedFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Write for OwnedFd {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(rustix::io::write(self, buf)?)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub enum FileDesc<'fd> {
    Owned(OwnedFd),
    Borrowed(BorrowedFd<'fd>),
}

impl<'fd> AsFd for FileDesc<'fd> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        match self {
            Self::Borrowed(fd) => fd.as_fd(),
            Self::Owned(fd) => fd.as_fd(),
        }
    }
}

impl AsRawFd for FileDesc<'_> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        match self {
            Self::Borrowed(fd) => fd.as_raw_fd(),
            Self::Owned(fd) => fd.as_raw_fd(),
        }
    }
}
