// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.

//! ## SimleFrame
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
//! Start Guard:    8 bits (0x01)
//! Payload Len:    16 bits
//! Payload Data:   Payload Len bytes
//! End Guard:      8 bits (0x17)
//! ```

use std::mem;

use super::{Frame, FrameBuilder};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct FrameGuard: u8 {
        const START     = 0b0000_0001;
        const END       = 0b0001_0111;
    }
}

#[derive(Clone)]
pub struct SimpleFrame {
    start_guard: FrameGuard,
    payload_len: u16,
    payload: Vec<u8>,
    end_guard: FrameGuard,
}

#[derive(Clone)]
pub struct SimpleFrameBuilder;

impl FrameBuilder for SimpleFrameBuilder {
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<dyn Frame>> {
        if buf.len() < 5 {
            return None;
        }

        let mut frame: SimpleFrame = Default::default();

        // Starting frame guard
        match FrameGuard::from_bits(buf[0]) {
            Some(start_guard) => {
                trace!("Start guard found");
                frame.start_guard = start_guard;
            }
            None => {
                error!(
                    "First byte was not expected start byte. Buffer corrupted?: {:#b}",
                    buf[0]
                );
            }
        }

        // Payload length
        let mask = 0xFFFFu16;
        let mut payload_len = ((buf[1] as u16) << 8) & mask;
        payload_len |= buf[2] as u16;
        frame.payload_len = payload_len;

        let payload_len = payload_len as usize;
        if buf.len() - 4 < payload_len {
            return None;
        }

        trace!("Payload length: {}", payload_len);

        // Payload data
        frame.payload.extend_from_slice(&buf[3..(payload_len + 3)]);

        // Ending frame guard
        match FrameGuard::from_bits(buf[payload_len + 3]) {
            Some(end_guard) => {
                trace!("End guard found");
                frame.end_guard = end_guard;
            }
            None => {
                error!(
                    "Last byte was not expected end byte. Buffer corrupted? {:#b}",
                    buf[payload_len + 3]
                );
                return None;
            }
        }

        // Remove frame from buffer
        let mut remainder = Vec::<u8>::with_capacity(buf.len() - frame.len_as_vec());
        remainder.extend_from_slice(&buf[frame.len_as_vec()..buf.len()]);
        mem::swap(buf, &mut remainder);

        return Some(Box::new(frame));
    }
}

impl SimpleFrame {
    /// Creates a new `SimpleFrame`
    pub fn new(buf: &[u8]) -> Self {
        SimpleFrame {
            start_guard: FrameGuard::START,
            payload_len: buf.len() as u16,
            payload: buf.to_vec(),
            end_guard: FrameGuard::END,
        }
    }
}

impl Frame for SimpleFrame {
    fn payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::<u8>::with_capacity(self.len_as_vec());
        buf.push(self.start_guard.bits());
        buf.push((self.payload_len >> 8) as u8);
        buf.push(self.payload_len as u8);
        buf.extend_from_slice(&self.payload[..]);
        buf.push(self.end_guard.bits());

        buf
    }

    fn len_as_vec(&self) -> usize {
        (self.payload_len + 4) as usize
    }

    fn as_mut_raw_erased(&self) -> *mut () {
        let dup = Box::new(self.clone());
        return Box::into_raw(dup) as *mut _ as *mut ();
    }
}

impl Default for SimpleFrame {
    fn default() -> SimpleFrame {
        SimpleFrame {
            start_guard: FrameGuard::START,
            payload_len: 0u16,
            payload: Vec::<u8>::new(),
            end_guard: FrameGuard::END,
        }
    }
}
