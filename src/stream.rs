// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::io::Error;
use std::os::unix::io::{RawFd, AsRawFd};


/// The `SRecv` trait allows for reading bytes from a source.
///
/// Each call to `recv` will attempt to pull bytes from the source
/// and place into the reader's internal buffer. When an `Ok(())`
/// result has been received, a complete message is capable of
/// being pulled out from the internal buffer with `drain_rx_queue`.
pub trait SRecv {
    /// Read bytes from the source into this Receiver's internal buffer.
    ///
    /// This call may be block or non-blocking depending on which
    /// stream implements this trait. When an `Ok(())` result is
    /// returned, at least one complete message has been received
    /// and can be pulled out from the internal buffer.
    ///
    /// # Errors
    /// This call will return an `Error` for any `std::io::Error`
    /// encountered during the read.
    fn recv(&mut self) -> Result<(), Error>;
    /// Drain the internal queue of recv'd messages, leaving the
    /// internal buffer empty.
    /// The length of the returned queue should be expected to be
    /// `1` when used on a blocking stream, and `>= 1` when used
    /// on a non-blocking stream.
    fn drain_rx_queue(&mut self) -> Vec<Vec<u8>>;
}

/// The `SSend` trait allows for the writing of bytes to a source.
///
/// Each call to `send` will attempt to write bytes to the source.
pub trait SSend {
    /// Attempt to write bytes to a source, returning how many bytes
    /// were written upon success.
    ///
    /// If the stream is in blocking mode, each call to `send` is
    /// expected to write directly to the source, followed by flushing.
    ///
    /// If the stream is in non-blocking mode, each call to `send` is
    /// expected to write directly to the source until the source returns
    /// `ErrorKind::WouldBlock`. At that point, the remaining bytes will
    /// be placed into an internal queue to be written first upon the next
    /// call.
    ///
    /// # Errors
    /// This call will return an `Error` for any `std::io::Error`
    /// encountered during the write.
    fn send(&mut self, buf: &[u8]) -> Result<usize, Error>;
}

/// The `StreamShutdown` is used for sutting down the stream source.
pub trait StreamShutdown {
    /// A call to this function will result in the stream source being shutdown
    /// and `Error` values being returned for any further I/O attempted.
    fn shutdown(&mut self) -> Result<(), Error>;
}

/// The `CloneStream` trait allows for specialized cloning of trait objects.
pub trait CloneStream {
    fn clone_stream(&self) -> Box<SStream>;
}

/// The `SStream` trait represents the entirety of methods each specialized
/// inner stream will have.
pub trait SStream: SRecv + SSend + StreamShutdown + CloneStream + AsRawFd {}


pub struct Stream {
    inner: Box<SStream>
}

impl Stream {
    /// Creates a new stream
    pub fn new(inner: Box<SStream>) -> Stream {
        Stream {
            inner: inner
        }
    }
}

impl SRecv for Stream {
    fn recv(&mut self) -> Result<(), Error> {
        self.inner.recv()
    }
    fn drain_rx_queue(&mut self) -> Vec<Vec<u8>> {
        self.inner.drain_rx_queue()
    }
}

impl SSend for Stream {
    fn send(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.inner.send(buf)
    }
}

impl StreamShutdown for Stream {
    fn shutdown(&mut self) -> Result<(), Error> {
        self.inner.shutdown()
    }
}

impl AsRawFd for Stream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Clone for Stream {
    fn clone(&self) -> Stream {
        Stream { inner: self.inner.clone_stream() }
    }
}

impl<T> CloneStream for T where T: 'static + Clone + SStream {
    fn clone_stream(&self) -> Box<SStream> {
        Box::new(self.clone())
    }
}


unsafe impl Send for Stream {}
unsafe impl Sync for Stream {}
