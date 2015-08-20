// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.


//! Bstream module.
//! This is a blocking stream designed to block on read/write until


use std::result::Result;
use std::net::TcpStream;
use std::io::{Read, Write, Error};

use super::readbuffer::ReadBuffer;


/// Represents the result of attempting a read on the underlying file descriptor
pub type ReadResult = Result<Vec<u8>, Error>;

/// Represents the result attempting a write on the underlying fild descriptor
pub type WriteResult = Result<(), Error>;


/// States the current stream can be in
#[derive(PartialEq, Clone)]
enum ReadState {
    /// Currently reading the payload length
    PayloadLen,
    /// Currently reading the payload
    Payload
}

pub struct Bstream {
    /// Current state
    state: ReadState,
    /// Underlying std::net::TcpStream
    stream: TcpStream,
    /// Message buffer
    buffer: ReadBuffer
}


impl Bstream {

    /// Returns a new Bstream
    pub fn new(stream: TcpStream) -> Bstream {
        Bstream {
            state: ReadState::PayloadLen,
            stream: stream,
            buffer: ReadBuffer::new()
        }
    }

    /// Performs a blocking read and returns when a complete message
    /// has been returned, or an error has occured
    pub fn read(&mut self) -> ReadResult {

        println!("simple stream read loop =============");

        loop {
            // Create a buffer for this specific read iteration
            let count = self.buffer.remaining();
            let mut buffer = Vec::<u8>::with_capacity(count as usize);
            unsafe { buffer.set_len(count as usize); }

            let result = self.stream.read(&mut buffer[..]);
            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let num_read = result.unwrap();
            for x in 0..num_read {
                self.buffer.push(buffer[x]);
            }

            if self.buffer.remaining() == 0 {
                if self.state == ReadState::PayloadLen {

                    println!("Payload length read complete");
                    println!("Printing buffer...");
                    let mut index = 0;
                    for byte in self.buffer.current_buffer().iter() {
                        println!("byte {}: {}", index, byte);
                        index += 1;
                    }

                    self.buffer.calc_payload_len();
                    let p_len = self.buffer.payload_len();

                    println!("Payload length: {}", p_len);

                    self.buffer.set_capacity(p_len);
                    self.state = ReadState::Payload;
                } else { // Payload completely read

                    println!("Payload read complete");
                    println!("Printing buffer...");
                    let mut index = 0;
                    for byte in self.buffer.current_buffer().iter() {
                        println!("byte {}: {}", index, byte);
                        index += 1;
                    }

                    self.buffer.reset();
                    self.state = ReadState::PayloadLen;
                    break;
                }
            }
        }

        println!("simple stream read loop finished =============");

        let mut buffer = self.buffer.drain_queue();

        println!("Queue drained");
        println!("buffer.len: {}", buffer.len());

        // This should always be .len() of 1
        // if it isn't - we're doing some bad stuff in here
        if buffer.len() != 1 {
            panic!("Error - Bstream.read() - Internal buffer was not equal to one...?")
        }

        match buffer.pop() {
            Some(buf) => Ok(buf),
            None => unimplemented!()
        }
    }

    /// Performs a blocking write operation and returns the complete buffer has
    /// been written, or an error has occured
    pub fn write(&mut self, buffer: &Vec<u8>) -> WriteResult {
        let mut plen_buf = [0u8; 2];
        plen_buf[0] = (buffer.len() as u16 & 0b1111_1111u16 << 8) as u8;
        plen_buf[1] = (buffer.len() as u16 & 0b1111_1111u16) as u8;

        let mut n_buffer = Vec::<u8>::with_capacity(buffer.len() + 2);
        n_buffer.push(plen_buf[0]);
        n_buffer.push(plen_buf[1]);

        for x in 0..buffer.len() {
            n_buffer.push(buffer[x]);
        }

        match self.stream.write_all(&n_buffer[..]) {
            Ok(()) => {
                let _ = self.stream.flush();
                Ok(())
            }
            Err(e) => Err(e)
        }
    }
}

impl Clone for Bstream {
    fn clone(&self) -> Bstream {
        Bstream {
            state: self.state.clone(),
            stream: self.stream.try_clone().unwrap(),
            buffer: self.buffer.clone()
        }
    }
}
