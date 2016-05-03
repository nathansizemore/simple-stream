// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! The simple-stream crate provides simple abstraction building blocks over any type that
//! implements [`std::io::Read`][std-io-read] + [`std::io::Write`][std-io-write].
//! Coupled with a [`FrameBuilder`][framebuilder], it provides built-in types for plain text
//! and secured streams with help from [rust-openssl][rust-openssl-repo]. It supports both
//! blocking and non-blocking modes.
//!
//! It works by handling all of the I/O on a frame based level. It includes a built-in framing
//! pattern and WebSocket based framing. The `simple_stream::frame` module includes traits for
//! using custom frames.
//!
//! ## Example Usage
//!
//! ```ignore
//! extern crate simple_stream as ss;
//!
//! use ss::frame::Frame;
//! use ss::frame::simple::{SimpleFrame, SimpleFrameBuilder};
//! use ss::{Socket, Plain, NonBlocking, SocketOptions};
//!
//!
//! fn main() {
//!     // tcp_stream is some connection established std::net::TcpStream
//!     //
//!     // Take ownership of the underlying fd to remove TcpStream's Drop being called now
//!     // that we're switching types.
//!     let fd = tcp_stream.into_raw_fd();
//!
//!     // Create a socket and set any POSIX based TCP/SOL_SOCKET options
//!     let mut socket = Socket::new(fd);
//!     socket.set_keepalive(true);
//!     socket.set_nonblocking();
//!
//!     // Create a plain text based stream that reads messages with SimpleFrame type
//!     let mut plain_stream = Plain::<Socket, SimpleFrameBuilder>::new(socket);
//!
//!     // Perform non-blocking read
//!     match plain_stream.nb_recv() {
//!         Ok(frames) => {
//!             // msgs is a Vec<Box<Frame>>
//!             for frame in frames.iter() {
//!                 // Do stuff with received things
//!             }
//!         }
//!         Err(e) => {
//!             // Error handling here
//!         }
//!     }
//!
//!     // Perform non-blocking write
//!     let frame = SimpleFrame::new(&some_buf[..]);
//!     plain_stream.nb_send(&frame).map_err(|e| {
//!         // Error handling here
//!     });
//! }
//! ```


#[macro_use]
extern crate log;
extern crate libc;
extern crate errno;
extern crate openssl;
#[macro_use]
extern crate bitflags;

use std::io::Error;

use frame::Frame;

pub use plain::*;
pub use socket::*;
pub use secure::*;

pub mod frame;
mod socket;
mod plain;
mod secure;


/// The `Blocking` trait provides method definitions for use with blocking streams.
pub trait Blocking {
    /// Performs a blocking read on the underlying stream until a complete Frame has been read
    /// or an `std::io::Error` has occurred.
    fn b_recv(&mut self) -> Result<Box<Frame>, Error>;
    /// Performs a blocking send on the underlying stream until a complete frame has been sent
    /// or an `std::io::Error` has occurred.
    fn b_send(&mut self, frame: &Frame) -> Result<(), Error>;
}

/// THe `NonBlocking` trait provides method definitions for use with non-blocking streams.
pub trait NonBlocking {
    /// Performs a non-blocking read on the underlying stream until `ErrorKind::WouldBlock` or an
    /// `std::io::Error` has occurred.
    ///
    /// # `simple_stream::Secure` notes
    ///
    /// Unlike its blocking counterpart, errors received on the OpenSSL level will be returned as
    /// `ErrorKind::Other` with various OpenSSL error information as strings in the description
    /// field of the `std::io::Error`.
    fn nb_recv(&mut self) -> Result<Vec<Box<Frame>>, Error>;
    /// Performs a non-blocking send on the underlying stream until `ErrorKind::WouldBlock` or an
    /// `std::io::Error` has occurred.
    ///
    /// # `simple_stream::Secure` notes
    ///
    /// Unlike its blocking counterpart, errors received on the OpenSSL level will be returned as
    /// `ErrorKind::Other` with various OpenSSL error information as strings in the description
    /// field of the `std::io::Error`.
    fn nb_send(&mut self, frame: &Frame) -> Result<(), Error>;
}
