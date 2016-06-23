// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! The `simple_stream::fame` module contains Trait definitions and built-in types for coupling
//! with a stream type to provide a structred way to send and receive message through streams.
//!
//! # Frame and FrameBuilder
//!
//! A `Frame` is the unit that streams operate on. Frames are used to ensure a complete piece of
//! information has been received and that complete pieces of information are sent without
//! fragmentation. A `FrameBuilder` is used by the stream types to construct a `Frame` from a
//! chunk of bytes.


pub use self::simple::SimpleFrame;
pub use self::websocket::WebSocketFrame;
pub use self::checksum32::Checksum32Frame;

mod simple;
mod websocket;
mod checksum32;

/// The Frame trait allows for type construction/destruction to/from a chunk of bytes.
pub trait Frame: Sync + Send {
    /// Transforms this type into a `Vec<u8>` in order to send through a stream.
    fn to_bytes(&self) -> Vec<u8>;
    /// Returns the paylaod data section of this `Frame`
    fn payload(&self) -> Vec<u8>;
    /// Returns the total length of the frame as if `to_bytes().len()` was called.
    fn len_as_vec(&self) -> usize;
    /// Returns a `*mut ()` to the underlying frame in order to cast to/from a specific
    /// type from the Trait Object returned from stream reads.
    ///
    /// It is up to the caller of this method to take care of the cleanup required of the specific
    /// type the pointer was cast to (E.g. by calling `Box::from_raw(ptr)').
    fn as_mut_raw_erased(&self) -> *mut ();
}

pub trait FrameBuilder {
    /// Given a `&mut Vec<u8>`, this function should return a Frame Trait Object, if possible,
    /// created from the bytes in `buf`. On success this method should remove all bytes that
    /// were used during the creation of the returned frame, from `buf`.
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<Frame>>;
}
