// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! simple-stream is a buffered stream wrapper over anything that implements
//! `std::io::Read` and `std::io::Write`. It works by buffering all reads and
//! checking the buffers against a `FrameBuilder`, which will inform the stream
//! that a complete `Frame` has been received, and removes it out of the buffer.
//!
//! The crate comes with a few types of Framing options, and provides both a plain
//! text and encrypted stream via [rust-openssl][rust-openssl-repo].
//!
//! ## Example Usage
//!
//! ```ignore
//! extern crate simple_stream as ss;
//!
//! use std::net::TcpStream;
//!
//! use ss::frame::{SimpleFrame, SimpleFrameBuilder};
//! use ss::{Plain, NonBlocking};
//!
//!
//! fn main() {
//!     // Create some non-blocking type that implements Read + Write
//!     let stream = TcpStream::connect("rust-lang.org:80").unwrap();
//!     stream.set_nonblocking(true).unwrap();
//!
//!     // Create a Plain Text stream that sends and receives messages in the
//!     // `SimpleFrame` format.
//!     let mut plain_stream = Plain::<TcpStream, SimpleFrameBuilder>::new(stream);
//!
//!     // Perform a non-blocking write
//!     let buf = vec!(1, 2, 3, 4);
//!     let frame = SimpleFrame::new(&buf[..]);
//!     match plain_stream.nb_send(&frame) {
//!         Ok(_) => { }
//!         Err(e) => println!("Error during write: {}", e)
//!     };
//!
//!     // Perform a non-blocking read
//!     match plain_stream.nb_recv() {
//!         Ok(frames) => {
//!             for _ in frames {
//!                 // Do stuff with received frames
//!             }
//!         }
//!         Err(e) => println!("Error during read: {}", e)
//!     };
//! }
//! ```
//!
//!
//! [rust-openssl-repo]: https://github.com/sfackler/rust-openssl


#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;
extern crate openssl;

use std::io::Error;

use frame::Frame;
pub use plain::*;
pub use secure::*;

pub mod frame;
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

/// The `NonBlocking` trait provides method definitions for use with non-blocking streams.
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
