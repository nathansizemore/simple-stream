// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use super::super::frame::{self, FrameState};
use super::super::stream::{SRecv, SSend, SStream, StreamShutdown};


#[derive(Clone)]
pub struct Plain<T> {
    inner: T,
    state: FrameState,
    buffer: Vec<u8>,
    scratch: Vec<u8>,
    tx_queue: Vec<Vec<u8>>,
    rx_queue: Vec<Vec<u8>>,
}

impl<T: Read + Write + AsRawFd + StreamShutdown> Plain<T> {
    pub fn new(stream: T) -> Plain<T> {
        Plain {
            inner: stream,
            state: FrameState::Start,
            buffer: Vec::with_capacity(3),
            scratch: Vec::new(),
            tx_queue: Vec::new(),
            rx_queue: Vec::new()
        }
    }
}

impl<T> Plain<T> {
    fn buf_with_scratch(&mut self, buf: &[u8], len: usize) -> Vec<u8> {
        let mut new_buf = Vec::<u8>::with_capacity(self.scratch.len() + len);
        for byte in self.scratch.iter() {
            new_buf.push(*byte);
        }
        self.scratch = Vec::<u8>::new();
        for x in 0..len {
            new_buf.push(buf[x]);
        }
        new_buf
    }

    fn read_for_frame_start(&mut self, buf: &[u8], offset: &mut usize, len: usize) {
        for _ in *offset..len {
            if buf[*offset] == frame::START {
                self.buffer.push(buf[*offset]);
                self.state = FrameState::PayloadLen;
                *offset += 1;
                break;
            }
            *offset += 1;
        }
    }

    fn read_payload_len(&mut self, buf: &[u8], offset: &mut usize, len: usize) {
        for _ in *offset..len {
            self.buffer.push(buf[*offset]);
            if self.buffer.len() == 3 {
                let len = self.payload_len() + 1;
                self.buffer.reserve_exact(len);
                self.state = FrameState::Payload;
                *offset += 1;
                break;
            }
            *offset += 1;
        }
    }

    fn read_payload(&mut self, buf: &[u8], offset: &mut usize, len: usize) {
        for _ in *offset..len {
            self.buffer.push(buf[*offset]);
            if self.buffer.len() == self.payload_len() + 3 {
                self.state = FrameState::End;
                *offset += 1;
                break;
            }
            *offset += 1;
        }
    }

    fn read_for_frame_end(&mut self, buf: &[u8], offset: usize, len: usize) -> Result<Vec<u8>, ()> {
        if offset < len {
            let expected_end_byte = buf[offset];
            if expected_end_byte == frame::END {
                let mut payload = Vec::<u8>::with_capacity(self.payload_len());
                for x in 3..self.buffer.len() {
                    payload.push(self.buffer[x]);
                }

                self.state = FrameState::Start;
                self.buffer = Vec::<u8>::with_capacity(3);

                // If there is anything left in buf, we need to put it in our
                // scratch space because we're exiting here
                let mut offset = offset;
                offset += 1;
                self.scratch = Vec::<u8>::with_capacity(len - offset);
                for x in offset..len {
                    self.scratch.push(buf[x]);
                }
                return Ok(payload);
            }

            // If we're here, the frame was wrong. Maybe our fault, who knows?
            // Either way, we're going to reset and try to start again from the start byte.
            // We need to dump whatever is left in the buffer into our scratch because it
            // might be in there?
            self.state = FrameState::Start;
            self.buffer = Vec::<u8>::with_capacity(3);
            self.scratch = Vec::<u8>::with_capacity(len - offset);
            for x in offset..len {
                self.scratch.push(buf[x]);
            }
        }
        Err(())
    }

    fn payload_len(&self) -> usize {
        let mask = 0xFFFFu16;
        let len = ((self.buffer[1] as u16) << 8) & mask;
        (len | self.buffer[2] as u16) as usize
    }

    fn vec_from_slice(&self, slice: &[u8]) -> Vec<u8> {
        let mut buf = Vec::<u8>::with_capacity(slice.len());
        for byte in slice.iter() {
            buf.push(*byte);
        }
        buf
    }
}

impl<T: Read + AsRawFd> SRecv for Plain<T> {
    fn recv(&mut self) -> Result<(), Error> {
        loop {
            let mut buf = Vec::<u8>::with_capacity(1024);
            unsafe {
                buf.set_len(1024);
            }
            let result = self.inner.read(&mut buf[..]);
            if result.is_err() {
                let err = result.unwrap_err();
                if err.kind() == ErrorKind::WouldBlock {
                    if self.rx_queue.len() > 0 {
                        return Ok(());
                    }
                }
                return Err(err);
            }
            let num_read = result.unwrap();

            buf = self.buf_with_scratch(&buf[..], num_read);
            let len = buf.len();
            let mut seek_pos = 0usize;

            if self.state == FrameState::Start {
                trace!("Reading for FrameState::Start");
                self.read_for_frame_start(&buf[..], &mut seek_pos, len);
            }

            if self.state == FrameState::PayloadLen {
                trace!("Reading for FrameState::PayloadLen");
                self.read_payload_len(&buf[..], &mut seek_pos, len);
            }

            if self.state == FrameState::Payload {
                trace!("Reading for FrameState::Payload");
                self.read_payload(&buf[..], &mut seek_pos, len);
            }

            if self.state == FrameState::End {
                trace!("Reading for FrameState::End");
                let result = self.read_for_frame_end(&buf[..], seek_pos, len);
                if result.is_ok() {
                    self.rx_queue.push(result.unwrap());
                }
            }
        }
    }

    fn drain_rx_queue(&mut self) -> Vec<Vec<u8>> {
        let buf = self.rx_queue.clone();
        self.rx_queue = Vec::new();
        buf
    }
}

impl<T: Write + AsRawFd> SSend for Plain<T> {
    fn send(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let mut total_written = 0usize;
        self.tx_queue.push(frame::from_slice(buf));
        for x in 0..self.tx_queue.len() {
            let b = self.tx_queue.remove(x);
            let result = self.inner.write(&b[..]);
            if result.is_err() {
                let err = result.unwrap_err();
                if err.kind() == ErrorKind::WouldBlock {
                    self.tx_queue.insert(x, b);
                    return Ok(total_written);
                }
                return Err(err);
            }

            let num_written = result.unwrap();
            total_written += num_written;
            if num_written < b.len() {
                let remainder = self.vec_from_slice(&b[(b.len() - num_written)..b.len()]);
                self.tx_queue.insert(x, remainder);
                return Ok(total_written);
            }
        }
        Ok(total_written)
    }
}

impl<T: StreamShutdown> StreamShutdown for Plain<T> {
    fn shutdown(&mut self) -> Result<(), Error> {
        self.inner.shutdown()
    }
}

impl<T: AsRawFd> AsRawFd for Plain<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl<T: 'static + Read + Write + AsRawFd + Clone + Send + StreamShutdown> SStream for Plain<T> {}
