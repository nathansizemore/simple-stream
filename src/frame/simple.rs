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
//! ```ignore
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | Frame Start   |  Payload Len                  |  Payload  |
//! +-----------------------------------------------------------+
//! |           Payload Data Continued          |   Frame End   |
//! +-----------------------------------------------------------+
//!
//! Start Frame:    8 bits, must be 0x01
//! Payload Len:    16 bits
//! Payload Data:   (Payload Len) bytes
//! End Frame:      8 bits, must be 0x17
//! ```


use std::mem;
use std::any::Any;
use std::default::Default;

use super::Frame;


bitflags! {
    flags FrameGuard: u8 {
        const START     = 0b0000_0001,
        const END       = 0b0000_1111
    }
}

#[derive(Clone)]
struct SimpleFrame {
    start_guard: FrameGuard,
    payload_len: u16,
    payload: Vec<u8>,
    end_guard: FrameGuard
}

impl Frame for SimpleFrame {

    #[allow(unused_variables)]
    fn new<T: Any>(buf: &[u8], args: &Vec<T>) -> Self {
        SimpleFrame {
            start_guard: START,
            payload_len: buf.len() as u16,
            payload: buf.to_vec(),
            end_guard: END
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let payload_len = self.payload_len as u16;
        let len: usize = self.payload_len as usize + 4;
        let mut buf = Vec::<u8>::with_capacity(len);

        buf.push(self.start_guard.bits());
        buf.push((payload_len >> 8) as u8);
        buf.push(payload_len as u8);
        buf.extend_from_slice(&self.payload[..]);
        buf.push(self.end_guard.bits());

        buf
    }

    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<Self>> {
        if buf.len() < 5 {
            return None;
        }

        let mut frame: SimpleFrame = Default::default();

        // Starting frame guard
        let first_byte = FrameGuard::from_bits(buf[0]).unwrap();
        if first_byte.bits() != START.bits() {
            error!("First byte was not expected start byte. Buffer corrupted?");
            return None;
        }
        frame.start_guard = first_byte;

        // Payload length
        let mut payload_len: u16;
        payload_len = (buf[1] as u16) << 8;
        payload_len |= buf[2] as u16;
        frame.payload_len = payload_len;

        let payload_len = payload_len as usize;
        if buf.len() < payload_len + 4 {
            return None;
        }

        // Payload data
        frame.payload.extend_from_slice(&buf[3..(payload_len + 3)]);

        // Ending frame guard
        let last_byte = FrameGuard::from_bits(buf[payload_len + 4]).unwrap();
        if last_byte.bits() != START.bits() {
            error!("Last byte was not expected end byte. Buffer corrupted?");
            return None;
        }
        frame.end_guard = last_byte;

        // Remove frame from buffer
        let mut remainder = Vec::<u8>::with_capacity(buf.len() - frame.len_as_vec());
        remainder.extend_from_slice(&buf[frame.len_as_vec()..buf.len()]);
        mem::swap(buf, &mut remainder);

        Some(Box::new(frame))
    }

    fn len_as_vec(&self) -> usize {
        self.payload_len as usize + 4
    }
}

impl Default for SimpleFrame {
    fn default() -> SimpleFrame {
        SimpleFrame {
            start_guard: START,
            payload_len: 0u16,
            payload: Vec::<u8>::new(),
            end_guard: END
        }
    }
}
