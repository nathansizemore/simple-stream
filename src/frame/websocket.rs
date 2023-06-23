// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.

//! The `frame::websocket` module provides [RFC-6465][rfc-6455] support for websocket based
//! streams. This module provides no support for the handshake part of the protocol, or any
//! smarts about handling fragmentation messages. It simply encodes/decodes complete websocket
//! frames.
//!
//! [rfc-6455]: https://tools.ietf.org/html/rfc6455

use std::{fmt, mem};

use super::{Frame, FrameBuilder};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct OpCode: u8 {
        const CONTINUATION  = 0b0000_0000;
        const TEXT          = 0b0000_0001;
        const BINARY        = 0b0000_0010;
        const CLOSE         = 0b0000_1000;
        const PING          = 0b0000_1001;
        const PONG          = 0b0000_1010;
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum FrameType {
    Control,
    Data,
}

#[derive(Clone, PartialEq, Eq)]
pub enum OpType {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

#[derive(Clone)]
struct Header {
    op_code: OpCode,
    mask: bool,
    payload_len: u64,
    masking_key: [u8; 4],
}

#[derive(Clone)]
struct Payload {
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct WebSocketFrame {
    frame_type: FrameType,
    header: Header,
    payload: Payload,
}

#[derive(Clone)]
pub struct WebSocketFrameBuilder;

impl FrameBuilder for WebSocketFrameBuilder {
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<dyn Frame>> {
        if buf.len() < 5 {
            return None;
        }

        let mut frame: WebSocketFrame = Default::default();

        // OpCode and FrameType
        const FIN_CLEAR_MASK: u8 = 0b0000_1111;
        let op_byte = buf[0] & FIN_CLEAR_MASK;
        match OpCode::from_bits(op_byte) {
            Some(op_code) => {
                if op_code == OpCode::CONTINUATION {
                    frame.frame_type = FrameType::Data;
                } else if op_code == OpCode::TEXT {
                    frame.frame_type = FrameType::Data;
                } else if op_code == OpCode::BINARY {
                    frame.frame_type = FrameType::Data;
                } else if op_code == OpCode::CLOSE {
                    frame.frame_type = FrameType::Control;
                } else if op_code == OpCode::PING {
                    frame.frame_type = FrameType::Control;
                } else if op_code == OpCode::PONG {
                    frame.frame_type = FrameType::Control;
                } else {
                    unreachable!();
                }

                frame.header.op_code = op_code;
            }
            None => {
                error!("Invalid OpCode bits: {:#b}", buf[0]);
                return None;
            }
        }

        trace!("{}", frame.op_type());

        // Payload masked (If from client, must always be true)
        let mask_bit = 0b1000_0000 & buf[1];
        frame.header.mask = mask_bit > 0;

        trace!("Frame masked: {}", frame.header.mask);

        // Payload data length
        let payload_len = 0b0111_1111 & buf[1];
        let mut next_offset: usize = 2;
        if payload_len <= 125 {
            frame.header.payload_len = payload_len as u64;
        } else if payload_len == 126 {
            let mut len = (buf[2] as u16) << 8;
            len |= buf[3] as u16;
            frame.header.payload_len = len as u64;
            next_offset = 4;
        } else {
            // We don't want to cause a panic
            if buf.len() < 10 {
                return None;
            }

            let mut len = (buf[2] as u64) << 56;
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

        trace!("Payload length: {}", frame.header.payload_len);

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
        frame
            .payload
            .data
            .extend_from_slice(&buf[next_offset..(len + next_offset)]);

        // Remove from buffer
        let mut remainder = Vec::<u8>::with_capacity(buf.len() - frame.len_as_vec());
        remainder.extend_from_slice(&buf[frame.len_as_vec()..buf.len()]);
        mem::swap(buf, &mut remainder);

        return Some(Box::new(frame));
    }
}

impl WebSocketFrame {
    pub fn new(buf: &[u8], frame_type: FrameType, op_type: OpType) -> WebSocketFrame {
        WebSocketFrame {
            frame_type,
            header: Header {
                op_code: match op_type {
                    OpType::Continuation => OpCode::CONTINUATION,
                    OpType::Text => OpCode::TEXT,
                    OpType::Binary => OpCode::BINARY,
                    OpType::Close => OpCode::CLOSE,
                    OpType::Ping => OpCode::PING,
                    OpType::Pong => OpCode::PONG,
                },
                mask: false,
                payload_len: buf.len() as u64,
                masking_key: [0u8; 4],
            },
            payload: Payload { data: buf.to_vec() },
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
            _ => unreachable!(),
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
        const FIN: u8 = 0b1000_0000;
        let op_code_with_fin = FIN | self.header.op_code.bits();
        buf.push(op_code_with_fin);

        // Mask and Payload len
        let mask_bit: u8 = if self.header.mask {
            0b1000_0000
        } else {
            0b0000_0000
        };
        let next_7_bits: u8 = if self.header.payload_len <= 125 {
            self.header.payload_len as u8
        } else if self.header.payload_len <= u16::MAX as u64 {
            126u8
        } else {
            127u8
        };
        let next_byte: u8 = mask_bit | next_7_bits;
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
                op_code: OpCode::CONTINUATION,
                mask: false,
                payload_len: 0u64,
                masking_key: [0u8; 4],
            },
            payload: Payload {
                data: Vec::<u8>::new(),
            },
        }
    }
}

impl fmt::Debug for FrameType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FrameType::Control => write!(f, "FrameType::Control"),
            FrameType::Data => write!(f, "FrameType::Data"),
        }
    }
}

impl fmt::Display for FrameType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FrameType::Control => write!(f, "FrameType::Control"),
            FrameType::Data => write!(f, "FrameType::Data"),
        }
    }
}

impl fmt::Debug for OpType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpType::Continuation => write!(f, "OpType::Continuation"),
            OpType::Text => write!(f, "OpType::Text"),
            OpType::Binary => write!(f, "OpType::Binary"),
            OpType::Close => write!(f, "OpType::Close"),
            OpType::Ping => write!(f, "OpType::Ping"),
            OpType::Pong => write!(f, "OpType::Pong"),
        }
    }
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpType::Continuation => write!(f, "OpType::Continuation"),
            OpType::Text => write!(f, "OpType::Text"),
            OpType::Binary => write!(f, "OpType::Binary"),
            OpType::Close => write!(f, "OpType::Close"),
            OpType::Ping => write!(f, "OpType::Ping"),
            OpType::Pong => write!(f, "OpType::Pong"),
        }
    }
}
