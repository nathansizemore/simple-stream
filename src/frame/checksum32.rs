// Copyright 2016 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! Provides a simple framing pattern with a 32-bit checksum validation
//! for each payload. This limits payload length to a max of `16,843,009` bytes.
//!
//! ```ignore
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                       Payload Length                          |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                        Payload Data                           |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                          Checksum                             |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!
//! Payload Length    Signed 32-bit integer Network Byte Order.
//! Payload Data      Payload Length bytes.
//! CHecksum          Sum of all bytes contained in Payload Data
//! ```
//!
//! [rfc-6455]: https://tools.ietf.org/html/rfc6455


use std::mem;
use std::default::Default;

use super::Frame;
use super::FrameBuilder;


#[derive(Clone)]
pub struct Checksum32Frame {
    payload_len: usize,
    payload: Vec<u8>,
    checksum: u32
}

#[derive(Clone)]
pub struct Checksum32FrameBuilder;
impl FrameBuilder for Checksum32FrameBuilder {
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<Frame>> {
        if buf.len() < 9 {
            return None;
        }

        let mut frame: Checksum32Frame = Default::default();

        // Payload length
        let mask = 0xFFFFFFFFu32;
        let mut payload_len: u32 = 0;
        payload_len |= ((buf[0] as u32) << 24) & mask;
        payload_len |= ((buf[1] as u32) << 16) & mask;
        payload_len |= ((buf[2] as u32) << 8) & mask;
        payload_len |= buf[3] as u32;

        let payload_len = payload_len as usize;
        frame.payload_len = payload_len;

        if buf.len() - 8 < payload_len {
            return None;
        }

        trace!("Payload length: {}", payload_len);

        let mut checksum: u32 = 0;
        for x in 4..(payload_len + 4) {
            let byte = buf[x];
            frame.payload.push(byte);
            checksum += byte as u32;
        }

        let mut maybe_checksum: u32 = 0;
        maybe_checksum |= ((buf[payload_len + 4 + 0] as u32) << 24) & mask;
        maybe_checksum |= ((buf[payload_len + 4 + 1] as u32) << 16) & mask;
        maybe_checksum |= ((buf[payload_len + 4 + 2] as u32) << 8) & mask;
        maybe_checksum |= buf[payload_len + 4 + 3] as u32;

        if maybe_checksum != checksum {
            error!("Checksum incorrect. Emptying passed buffer");
            *buf = Vec::new();
            return None;
        }

        frame.checksum = checksum;
        let mut remainder = Vec::<u8>::with_capacity(buf.len() - frame.len_as_vec());
        remainder.extend_from_slice(&buf[frame.len_as_vec()..buf.len()]);
        mem::swap(buf, &mut remainder);

        Some(Box::new(frame))
    }
}

impl Checksum32Frame {
    pub fn new(buf: &[u8]) -> Self {
        let len = buf.len();
        let mut checksum: u32 = 0;
        let mut v = Vec::<u8>::with_capacity(len);

        for x in 0..len {
            let byte = buf[x];
            v.push(byte);
            checksum += byte as u32;
        }

        Checksum32Frame {
            payload_len: len,
            payload: v,
            checksum: checksum
        }
    }
}

impl Frame for Checksum32Frame {
    fn payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::<u8>::with_capacity(self.len_as_vec());
        buf.push((self.payload_len >> 24) as u8);
        buf.push((self.payload_len >> 16) as u8);
        buf.push((self.payload_len >> 8) as u8);
        buf.push(self.payload_len as u8);
        buf.extend_from_slice(&self.payload[..]);
        buf.push((self.checksum >> 24) as u8);
        buf.push((self.checksum >> 16) as u8);
        buf.push((self.checksum >> 8) as u8);
        buf.push(self.checksum as u8);

        buf
    }

    fn len_as_vec(&self) -> usize {
        self.payload_len + 8
    }

    fn as_mut_raw_erased(&self) -> *mut () {
        let dup = Box::new(self.clone());
        return Box::into_raw(dup) as *mut _ as *mut ();
    }
}

impl Default for Checksum32Frame {
    fn default() -> Checksum32Frame {
        Checksum32Frame {
            payload_len: 0,
            payload: Vec::new(),
            checksum: 0
        }
    }
}
