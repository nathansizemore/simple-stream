// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.


//! NbetStream module.
//! This is a Non-blocking file descriptor stream designed to be used with
//! Linux epoll in EdgeTriggered mode.


use std::{mem, ptr};
use std::result::Result;
use std::net::TcpStream;
use std::os::unix::io::{RawFd, AsRawFd};

use super::libc;
use super::errno::errno;
use super::libc::{size_t, c_void, c_int, ssize_t};
use super::readbuffer::ReadBuffer;

use self::util::*;

pub mod util;


extern "C" {
    fn read(fd: c_int, buffer: *mut c_void, count: size_t) -> ssize_t;
    fn write(fd: c_int, buffer: *const c_void, cout: size_t) -> ssize_t;
}


/// Represents the result of trying to create a new NbetStream
pub type CreateResult = Result<NbetStream, FnctlError>;

/// Represents the result of attempting a read on the underlying file descriptor
pub type ReadResult = Result<(), ReadError>;

/// Represents the result attempting a write on the underlying fild descriptor
pub type WriteResult = Result<u16, WriteError>;


/// States the current stream can be in
#[derive(PartialEq, Clone)]
enum ReadState {
    /// Currently reading the payload length
    PayloadLen,
    /// Currently reading the payload
    Payload
}

/// Struct representing a simple messaging protocol over Tcp sockets
pub struct NbetStream {
    /// Current state
    state: ReadState,
    /// Underlying std::net::TcpStream
    stream: TcpStream,
    /// Message buffer
    buffer: ReadBuffer
}


impl NbetStream {

    /// Attempts to create a new NbetStream from a TcpStream
    pub fn new(stream: TcpStream) -> CreateResult {
        let fd = stream.as_raw_fd();
        let response;
        unsafe {
            response = libc::fcntl(
                fd,
                libc::F_SETFL,
                libc::O_NONBLOCK);
        }

        if response < 0 {
            let errno = errno().0 as i32;
            return match errno {
                libc::EAGAIN     => Err(FnctlError::EAGAIN),
                libc::EBADF      => Err(FnctlError::EBADF),
                libc::EDEADLK    => Err(FnctlError::EDEADLK),
                libc::EFAULT     => Err(FnctlError::EFAULT),
                libc::EINTR      => Err(FnctlError::EINTR),
                libc::EINVAL     => Err(FnctlError::EINVAL),
                libc::EMFILE     => Err(FnctlError::EMFILE),
                libc::ENOLCK     => Err(FnctlError::ENOLCK),
                libc::EPERM      => Err(FnctlError::EPERM),
                _ => panic!("Unexpected errno: {}", errno)
            };
        }

        Ok(NbetStream {
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
            let count = self.buffer.remaining();
            let result = self.read_num_bytes(count);
            if result.is_err() {
                let err = result.unwrap_err();
                match err {
                    ReadError::EAGAIN => {
                        return Ok(())
                    }
                    _ => return Err(err)
                }
            }

            if self.buffer.remaining() == 0 {
                if self.state == ReadState::PayloadLen {
                    self.buffer.calc_payload_len();
                    let p_len = self.buffer.payload_len();
                    self.buffer.set_capacity(p_len);
                    self.state = ReadState::Payload;
                } else { // Payload completely read
                    self.buffer.reset();
                    self.state = ReadState::PayloadLen;
                }
            }
        }
    }

    /// Attempts to read num bytes from the underlying file descriptor.
    fn read_num_bytes(&mut self, num: u16) -> ReadResult {
        let fd = self.stream.as_raw_fd();

        // Create a buffer, size num
        let buffer;
        unsafe {
            buffer = libc::calloc(num as size_t, mem::size_of::<u8>() as size_t);
        }

        // Ensure system gave up dynamic memory
        if buffer.is_null() {
            return Err(ReadError::ENOMEM)
        }

        // Attempt to read available data into buffer
        let num_read;
        unsafe {
            num_read = read(fd, buffer, num as size_t);
        }

        // Return on error
        if num_read < 0 {
            unsafe { libc::free(buffer); }
            let errno = errno().0 as i32;
            return match errno {
                libc::ENOMEM         => Err(ReadError::ENOMEM),
                libc::EBADF          => Err(ReadError::EBADF),
                libc::EFAULT         => Err(ReadError::EFAULT),
                libc::EINTR          => Err(ReadError::EINTR),
                libc::EINVAL         => Err(ReadError::EINVAL),
                libc::EIO            => Err(ReadError::EIO),
                libc::EISDIR         => Err(ReadError::EISDIR),
                libc::ECONNRESET     => Err(ReadError::ECONNRESET),

                // These two constants differ between OSes, so we can't use a match clause
                x if x == libc::EAGAIN || x == libc::EWOULDBLOCK => Err(ReadError::EAGAIN),

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

    /// Attempts to write the buffer to the underlying file descriptor
    pub fn write(&mut self, buffer: &Vec<u8>) -> WriteResult {
        let mut plen_buf = [0u8; 2];
        let plen = buffer.len() as u16;
        plen_buf[0] = (plen >> 8) as u8;
        plen_buf[1] = plen as u8;

        let mut n_buffer = Vec::<u8>::with_capacity(buffer.len() + 2);
        n_buffer.push(plen_buf[0]);
        n_buffer.push(plen_buf[1]);

        for x in 0..buffer.len() {
            n_buffer.push(buffer[x]);
        }

        self.write_bytes(&mut n_buffer)
    }

    /// Attempts to write the passed buffer to the underlying file descriptor
    fn write_bytes(&mut self, buffer: &mut Vec<u8>) -> WriteResult {
        let fd = self.stream.as_raw_fd();

        let num_written;
        unsafe {
            let buf_slc = &buffer[..];
            let buf_ptr = buf_slc.as_ptr();
            let v_ptr: *const c_void = mem::transmute(buf_ptr);
            num_written = write(fd,
                v_ptr,
                buffer.len() as size_t);
        }

        if num_written < 0 {
            let errno = errno().0 as i32;
            return match errno {
                libc::EBADF          => Err(WriteError::EBADF),
                libc::EDESTADDRREQ   => Err(WriteError::EDESTADDRREQ),
                libc::EDQUOT         => Err(WriteError::EDQUOT),
                libc::EFAULT         => Err(WriteError::EFAULT),
                libc::EFBIG          => Err(WriteError::EFBIG),
                libc::EINTR          => Err(WriteError::EINTR),
                libc::EINVAL         => Err(WriteError::EINVAL),
                libc::EIO            => Err(WriteError::EIO),
                libc::ENOSPC         => Err(WriteError::ENOSPC),
                libc::EPIPE          => Err(WriteError::EPIPE),
                libc::ECONNRESET     => Err(WriteError::ECONNRESET),

                // These two constants can have the same value on some systems,
                // but different values on others, so we can't use a match
                // clause
                x if x == libc::EAGAIN || x == libc::EWOULDBLOCK =>
                    Err(WriteError::EAGAIN),

                _ => panic!("Unknown errno during write: {}", errno),
            }
        }

        Ok(num_written as u16)
    }

    /// Returns the underlying file descriptor
    pub fn raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }

    /// Returns a mutable reference to the internal ReadBuffer
    pub fn buffer_as_mut(&mut self) -> &mut ReadBuffer {
        &mut self.buffer
    }
}


impl Clone for NbetStream {
    fn clone(&self) -> NbetStream {
        NbetStream {
            state: self.state.clone(),
            stream: self.stream.try_clone().unwrap(),
            buffer: self.buffer.clone()
        }
    }
}
