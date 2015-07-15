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

    /// Performs a read on the underlying fd. Places all received messages into
    /// the queue in the ReadBuffer. Reads until EAGAIN is hit.
    pub fn read(&mut self) -> ReadResult {
        // We need to loop until EAGAIN is hit from read_num_bytes.
        // Epoll is set to EdgeTrigged mode, which will let us known when there
        // is data to be read on the file descriptor, but if we do not clear it
        // all in this run we will lose whatever we do not grab.
        loop {
            if self.state == ReadState::PayloadLen {
                let result = self.read_num_bytes(self.buffer.remaining());
                if result.is_err() {
                    let err = result.unwrap_err();
                    match err {
                        ReadError::EAGAIN => {
                            return Ok(())
                        }
                        ReadError::EWOULDBLOCK => {
                            return Ok(())
                        }
                        _ => return Err(err)
                    }
                }

                // Have we finished reading payload len?
                if self.buffer.remaining() == 0 {
                    self.buffer.set_payload_len();
                    self.buffer.set_capacity(self.buffer.payload_len());
                } else {
                    continue;
                }
            }
        }
    }

    ///
    fn read_payload_len() -> ReadResult {

    }

    ///
    fn read_payload() -> ReadResult {

    }

    ///
    fn read_num_bytes(&mut self, num: u16) -> ReadResult {
        let fd = self.stream.as_raw_fd();

        // Create a buffer, size num
        let mut buffer;
        unsafe {
            buffer = libc::calloc(num as size_t,
                mem::size_of::<u8>() as size_t);
        }

        // Ensure system gave up dynamic memory
        if buffer.is_null() {
            return Err(ReadError::ENOMEM)
        }

        // Attempt to read available data into buffer
        let mut num_read;
        unsafe {
            num_read = read(fd, buffer, count as size_t);
        }

        // Return on error
        if num_read < 0 {
            unsafe { libc::free(buffer); }
            let errno = errno().0 as i32;
            return match errno {
                posix88::ENOMEM         => Err(ReadError::ENOMEM),
                posix88::EAGAIN         => Err(ReadError::EAGAIN),
                posix88::EWOULDBLOCK    => Err(ReadError::EWOULDBLOCK),
                posix88::EBADF          => Err(ReadError::EBADF),
                posix88::EFAULT         => Err(ReadError::EFAULT),
                posix88::EINTR          => Err(ReadError::EINTR),
                posix88::EINVAL         => Err(ReadError::EINVAL),
                posix88::EIO            => Err(ReadError::EIO),
                posix88::EISDIR         => Err(ReadError::EISDIR),
                _ => panic!("Unexpected errno during read: {}", errno)
            };
        }

        // Check for EOF
        if num_read == 0 {
            unsafe { libc::free(buffer); }
            return Err(ReadError::EOF);
        }

        // Add bytes to msg buffer
        for x in 0..num_read as isize {
            unsafe {
                self.buffer.push(ptr::read(buffer.offset(x)) as u8);
            }
        }

        // Free buffer and return Ok
        unsafe { libc::free(buffer); }
        Ok(())
    }
}
