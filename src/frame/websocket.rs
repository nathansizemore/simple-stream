// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::u16;
use std::default::Default;

use super::{Frame, FrameBuilder};


bitflags! {
    flags OpCode: u8 {
        const CONTINUATION  = 0b0000_0000,
        const TEXT          = 0b0000_0001,
        const BINARY        = 0b0000_0010,
        const CLOSE         = 0b0000_1000,
        const PING          = 0b0000_1001,
        const PONG          = 0b0000_1010
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum FrameType {
    Control,
    Data
}

#[derive(Clone, PartialEq, Eq)]
pub enum OpType {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong
}

#[derive(Clone)]
struct Header {
    op_code: OpCode,
    mask: bool,
    payload_len: u64,
    masking_key: [u8; 4]
}

#[derive(Clone)]
struct Payload {
    data: Vec<u8>
}

#[derive(Clone)]
pub struct WebSocketFrame {
    frame_type: FrameType,
    header: Header,
    payload: Payload
}

#[derive(Clone)]
pub struct WebSocketFrameBuilder;
impl FrameBuilder for WebSocketFrameBuilder {
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<Frame>> {
        if buf.len() < 5 {
            trace!("Buffer length is less than 5, not worth trying...");
            return None;
        }

        let mut frame: WebSocketFrame = Default::default();

        // OpCode and FrameType
        match OpCode::from_bits(buf[0]) {
            Some(op_code) => {
                if op_code.contains(CONTINUATION) {
                    frame.frame_type = FrameType::Data;
                    frame.header.op_code = CONTINUATION;
                } else if op_code.contains(TEXT) {
                    frame.frame_type = FrameType::Data;
                    frame.header.op_code = TEXT;
                } else if op_code.contains(BINARY) {
                    frame.frame_type = FrameType::Data;
                    frame.header.op_code = BINARY;
                } else if op_code.contains(CLOSE) {
                    frame.frame_type = FrameType::Control;
                    frame.header.op_code = CLOSE;
                } else if op_code.contains(PING) {
                    frame.frame_type = FrameType::Control;
                    frame.header.op_code = PING;
                } else if op_code.contains(PONG) {
                    frame.frame_type = FrameType::Control;
                    frame.header.op_code = PONG;
                }
            }
            None => {
                error!("Invalid OpCode bits: {:#b}", buf[0]);
                return None;
            }
        }

        // Payload masked (If from client, must always be true)
        let mask_bit = 0b1000_0000 & buf[1];
        frame.header.mask = mask_bit > 0;

        // Payload data length
        let mut next_offset: usize = 3;
        let payload_len = 0b0111_1111 & buf[2];
        if payload_len <= 125 {
            frame.header.payload_len = payload_len as u64;
        } else if payload_len == 126 {
            let mut len = (buf[3] as u16) << 8;
            len |= buf[4] as u16;
            frame.header.payload_len = len as u64;
            next_offset = 5;
        } else {
            // We don't want to cause a panic
            if buf.len() < 10 {
                return None;
            }

            let mut len = (buf[3] as u64) << 56;
            len |= (buf[3] as u64) << 48;
            len |= (buf[4] as u64) << 40;
            len |= (buf[5] as u64) << 32;
            len |= (buf[6] as u64) << 24;
            len |= (buf[7] as u64) << 16;
            len |= (buf[8] as u64) << 8;
            len |= buf[9] as u64;
            frame.header.payload_len = len;
            next_offset = 10;
        }

        // Optional masking key
        if frame.header.mask {
            if buf.len() <= next_offset + 4 {
                return None;
            }
            frame.header.masking_key[0] = buf[next_offset];
            frame.header.masking_key[1] = buf[next_offset + 1];
            frame.header.masking_key[2] = buf[next_offset + 2];
            frame.header.masking_key[3] = buf[next_offset + 3];
            next_offset += 4;
        }

        if buf.len() < next_offset + frame.header.payload_len as usize {
            return None;
        }

        // Payload data
        let len = frame.header.payload_len as usize;
        frame.payload.data.extend_from_slice(&buf[next_offset..len]);

        return Some(Box::new(frame));
    }
}

impl WebSocketFrame {
    pub fn new(buf: &[u8], frame_type: FrameType, op_type: OpType) -> WebSocketFrame {
        WebSocketFrame {
            frame_type: frame_type,
            header: Header {
                op_code: match op_type {
                    OpType::Continuation => CONTINUATION,
                    OpType::Text => TEXT,
                    OpType::Binary => BINARY,
                    OpType::Close => CLOSE,
                    OpType::Ping => PING,
                    OpType::Pong => PONG,
                },
                mask: false,
                payload_len: buf.len() as u64,
                masking_key: [0u8; 4]
            },
            payload: Payload {
                data: buf.to_vec()
            }
        }
    }

    pub fn op_type(&self) -> OpType {
        match self.header.op_code {
            CONTINUATION => OpType::Continuation,
            TEXT => OpType::Text,
            BINARY => OpType::Binary,
            CLOSE => OpType::Close,
            PING => OpType::Ping,
            PONG => OpType::Pong,
            _ => unreachable!()
        }
    }

    pub fn frame_type(&self) -> FrameType {
        self.frame_type.clone()
    }

    pub fn is_masked(&self) -> bool {
        self.header.mask
    }

    pub fn payload_unmasked(&self) -> Vec<u8> {
        let len = self.payload.data.len();
        let mut buf = Vec::<u8>::with_capacity(len);
        for x in 0..len {
            buf.push(self.payload.data[x] ^ self.header.masking_key[x % 4]);
        }

        buf
    }
}

impl Frame for WebSocketFrame {
    fn payload(&self) -> Vec<u8> {
        if self.header.mask {
            self.payload_unmasked()
        } else {
            self.payload.data.clone()
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::<u8>::with_capacity(self.len_as_vec());

        // OpCode
        buf.push(self.header.op_code.bits());

        // Mask and Payload len
        let mask_bit: u8 = if self.header.mask {
            0b1000_000
        } else {
            0b0000_000
        };
        let next_7_bits: u8 = if self.header.payload_len <= 125 {
            self.header.payload_len as u8
        } else if self.header.payload_len <= u16::MAX as u64 {
            126u8
        } else {
            127u8
        };
        let next_byte: u8 = mask_bit & next_7_bits;
        buf.push(next_byte);

        // Optional payload len
        if next_byte == 126 {
            buf.push(((self.header.payload_len as u16) >> 8) as u8);
            buf.push(self.header.payload_len as u8);
        } else if next_byte == 127 {
            buf.push((self.header.payload_len >> 56) as u8);
            buf.push((self.header.payload_len >> 48) as u8);
            buf.push((self.header.payload_len >> 40) as u8);
            buf.push((self.header.payload_len >> 32) as u8);
            buf.push((self.header.payload_len >> 24) as u8);
            buf.push((self.header.payload_len >> 16) as u8);
            buf.push((self.header.payload_len >> 8) as u8);
            buf.push(self.header.payload_len as u8);
        }

        // Optional masking key
        if self.header.mask {
            buf.push(self.header.masking_key[0]);
            buf.push(self.header.masking_key[1]);
            buf.push(self.header.masking_key[2]);
            buf.push(self.header.masking_key[3]);
        }

        // Payload data
        buf.extend_from_slice(&self.payload.data[..]);

        buf
    }

    fn len_as_vec(&self) -> usize {
        let mut len = 0usize;

        // OpCode
        len += 1;

        // Mask and paylaod length
        len += 1;

        // Extended Payload length
        if self.header.payload_len > 125 && self.header.payload_len < u16::MAX as u64 {
            len += 2;
        } else if self.header.payload_len > u16::MAX as u64 {
            len += 8;
        }

        // Optional masking key
        if self.header.mask {
            len += 4;
        }

        // Payload data
        len += self.header.payload_len as usize;

        len
    }

    fn as_mut_raw_erased(&self) -> *mut () {
        let dup = Box::new(self.clone());
        return Box::into_raw(dup) as *mut _ as *mut ();
    }
}

impl Default for WebSocketFrame {
    fn default() -> WebSocketFrame {
        WebSocketFrame {
            frame_type: FrameType::Control,
            header: Header {
                op_code: CONTINUATION,
                mask: false,
                payload_len: 0u64,
                masking_key: [0u8; 4]
            },
            payload: Payload {
                data: Vec::<u8>::new()
            }
        }
    }
}
