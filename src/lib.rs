// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! The simple-stream crate provides a simple framing protocol over any type that implements the
//! [`SStream`][e] trait. It also provides built-in support for blocking and non-blocking streams over
//! Unix file descriptors in both plain-text and SSL through [rust-openssl][r].
//!
//! ## Usage
//!
//! ```no_run
//! extern crate simple_stream as ss;
//!
//! use std::net::TcpStream;
//! use std::os::unix::io::IntoRawFd;
//!
//! use ss::{Socket, Stream, SSend, SRecv, StreamShutdown};
//! use ss::nonblocking::plain::Plain;
//!
//!
//! fn main() {
//!     // Starting with an established TcpStream
//!     let tcp_stream = TcpStream::connect("127.0.0.1").unwrap();
//!
//!     // Create a socket that takes ownership of the underlying fd
//!     let socket = Socket::new(tcp_stream.into_raw_fd());
//!
//!     // Pick a built-in stream type to wrap the socket
//!     let plain_text = Plain::new(socket);
//!
//!     // Create a Stream
//!     let mut stream = Stream::new(Box::new(plain_text));
//!
//!     // Write a thing
//!     let buffer = "ping".as_bytes();
//!     match stream.send(&buffer[..]) {
//!         Ok(num_written) => println!("Wrote: {} bytes", num_written),
//!         Err(e) => {
//!             println!("Error during write: {}", e);
//!             stream.shutdown().unwrap();
//!         }
//!     }
//!
//!     // Receive all the things
//!     match stream.recv() {
//!         Ok(()) => {
//!             let mut queue = stream.drain_rx_queue();
//!             for msg in queue.drain(..) {
//!                 println!("Received: {}", String::from_utf8(msg).unwrap());
//!             }
//!         }
//!         Err(e) => {
//!             println!("Error during read: {}", e);
//!             stream.shutdown().unwrap();
//!         }
//!     }
//! }
//! ```
//!
//! [e]: https://nathansizemore.github.io/simple-stream/simple_stream/stream/trait.SStream.html
//! [r]: https://github.com/sfackler/rust-openssl

#[macro_use]
extern crate log;
extern crate libc;
extern crate errno;
extern crate openssl;


pub use stream::*;
pub use socket::*;

pub mod blocking;
pub mod nonblocking;

mod frame;
mod stream;
mod socket;
