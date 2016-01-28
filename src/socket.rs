// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::mem;
use std::ops::Drop;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use libc;
use errno::errno;
use libc::{c_int, c_void};

use stream::StreamShutdown;


/// The `TcpOptions` trait allows for various TCP setting used in syscalls
/// throughout Unix-like kernels.
pub trait TcpOptions {
    /// Sets the `SO_KEEPALIVE` flag to `keepalive` on this socket.
    fn set_tcp_keepalive(&mut self, keepalive: bool) -> Result<(), Error>;
    /// Sets the `TCP_NODELAY` flag to `nodelay` on this socket.
    fn set_tcp_nodelay(&mut self, nodelay: bool) -> Result<(), Error>;
}


#[derive(Clone, Eq, PartialEq)]
pub struct Socket {
    fd: RawFd,
}

impl Socket {
    /// Creates a new socket with assumed ownership of `fd`
    pub fn new(fd: RawFd) -> Socket {
        Socket {
            fd: fd
        }
    }
}

impl TcpOptions for Socket {
    fn set_tcp_keepalive(&mut self, keepalive: bool) -> Result<(), Error> {
        let optval: c_int = match keepalive {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_KEEPALIVE,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_tcp_nodelay(&mut self, nodelay: bool) -> Result<(), Error> {
        const SOL_TCP: c_int = 6;

        let optval: c_int = match nodelay {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             SOL_TCP,
                             libc::TCP_NODELAY,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.len() < 1 {
            return Err(Error::new(ErrorKind::Other, "Invalid buffer"));
        }

        let result = unsafe { libc::read(self.fd, buf as *mut _ as *mut c_void, buf.len()) };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        if result == 0 {
            return Err(Error::new(ErrorKind::Other, "EOF"));
        }

        Ok(result as usize)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let result = unsafe {
            libc::write(self.fd, buf as *const _ as *const c_void, buf.len())
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(result as usize)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl StreamShutdown for Socket {
    fn shutdown(&mut self) -> Result<(), Error> {
        let result = unsafe {
            libc::close(self.fd)
        };
        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}
