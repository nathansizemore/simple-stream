// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible
// With Secondary Licenses", as defined by
// the Mozilla Public License, v. 2.0.


//! SimpleStream crate.


extern crate libc;
extern crate errno;

use std::{mem, ptr, fmt};
use std::result::Result;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

use self::errno::errno;
use self::libc::consts::os::posix88;
use self::libc::{size_t, c_void, c_int, ssize_t};

use util::*;
use message::Message;
use readbuffer::ReadBuffer;

mod message;
mod readbuffer;
pub mod util;



extern "C" {
    fn read(fd: c_int, buffer: *mut c_void, count: size_t) -> ssize_t;
    fn write(fd: c_int, buffer: *const c_void, cout: size_t) -> ssize_t;
}


/// Represents the result of trying to create a new SimpleStream
pub type CreateResult = Result<StreamStream, FcntlError>;

/// Represents the result of attempting a read on the underlying file descriptor
pub type ReadResult = Result<(), ReadError>;

/// Represents the result attempting a write on the underlying fild descriptor
pub type WriteResult = Result<u16, WriteError>;


/// States the current stream can be in
#[derive(PartialEq, Clone)]
pub enum ReadState {
    /// Currently reading the payload length
    PayloadLen,
    /// Currently reading the payload
    Payload
}

/// Struct representing a simple messaging protocol over Tcp sockets
#[derive(Clone)]
pub struct SimpleStream {
    /// Current state
    state: ReadState,
    /// Underlying std::net::TcpStream
    stream: TcpStream,
    /// Message buffer
    buffer: ReadBuffer
}


impl SimpleStream {

    /// Attempts to create a new SimpleStream from a TcpStream
    pub fn new(stream: TcpStream) -> CreateResult {
        let fd = stream.as_raw_fd();
        let mut response;
        unsafe {
            response = libc::fcntl(
                fd,
                libc::consts::os::posix01::F_SETFL,
                libc::consts::os::extra::O_NONBLOCK);
        }

        if response < 0 {
            let errno = errno().0 as i32;
            return match errno {
                posix88::EAGAIN     => Err(FnctlError::EAGAIN),
                posix88::EBADF      => Err(FnctlError::EBADF),
                posix88::EDEADLK    => Err(FnctlError::EDEADLK),
                posix88::EFAULT     => Err(FnctlError::EFAULT),
                posix88::EINTR      => Err(FnctlError::EINTR),
                posix88::EINVAL     => Err(FnctlError::EINVAL),
                posix88::EMFILE     => Err(FnctlError::EMFILE),
                posix88::ENOLCK     => Err(FnctlError::ENOLCK),
                posix88::EPERM      => Err(FnctlError::EPERM),
                _ => panic!("Unexpected errno: {}", errno)
            };
        }

        Ok(SimpleStream {
            state: ReadState::PayloadLen,
            stream: stream,
            buffer: ReadBuffer::new()
        })
    }

    /// Performs a read on the underlying fd. Places all received messages into the
    /// queue in the ReadBuffer. Reads until EAGAIN is hit.
    pub fn read(&mut self) -> ReadResult {
        // We need to loop until EAGAIN is hit from read_num_bytes.
        // Epoll is set to EdgeTrigged mode, which will let us known when there is data
        // to be read on the file descriptor, but if we do not clear it all in this run
        // we will lose whatever we do not grab.
        loop {
            if self.state == ReadState::PayloadLen {
                // Do we need to reset our internal buffer?
                if self.buffer.frame_complete() {
                    let result = self.read_payload_len();
                    if result.is_err() {
                        let err = result.unwrap_err();
                        match result.unwrap_err() {
                            ReadError::EAGAIN => {

                            }
                            ReadError::EWOULDBLOCK => {

                            }
                        }
                    }
                }
            }
        }
    }

    ///
    fn read_payload_len() -> ReadResult {

    }
}
