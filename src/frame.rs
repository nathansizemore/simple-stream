// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! The frame module provides a structred way to send and receive
//! message through streams.
//!
//! ## Data Framing
//!
//! ~~~
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | Frame Start |  Payload Len                  |   Payload   |
//! +-----------------------------------------------------------+
//! |           Payload Data Continued            |  Frame End  |
//! +-----------------------------------------------------------+
//!
//! Start Frame:    8 bits, must be 0x01
//! Payload Len:    16 bits
//! Payload Data:   (Payload Len) bytes
//! End Frame:      8 bits, must be 0x17
//! ~~~


use std::fmt;


/// Indicates start of frame
pub const START:    u8 = 0x01;
/// Indicates end of frame
pub const END:      u8 = 0x17;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FrameState {
    /// The stream is currently reading for start byte
    Start,
    /// The stream is currently reading for payload length
    PayloadLen,
    /// The stream is currently reading the payload
    Payload,
    /// The stream is currently reading for the end byte
    End,
}

impl fmt::Display for FrameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FrameState::Start => "Start".fmt(f),
            FrameState::PayloadLen => "PayloadLen".fmt(f),
            FrameState::Payload => "Payload".fmt(f),
            FrameState::End => "End".fmt(f),
        }
    }
}

pub fn from_slice(slice: &[u8]) -> Vec<u8> {
    let len = slice.len() as u16;
    let mut buf = Vec::<u8>::with_capacity(slice.len() + 4);
    buf.push(START);
    buf.push((len >> 8) as u8);
    buf.push(len as u8);
    for byte in slice.iter() {
        buf.push(*byte);
    }
    buf.push(END);
    buf
}
