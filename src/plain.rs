// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::mem;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use frame;

use super::{Blocking, NonBlocking};


const BUF_SIZE: usize = 1024;


#[derive(Clone)]
pub struct Plain<T: Read + Write> {
    inner: T,
    rx_buf: Vec<u8>,
    tx_buf: Vec<u8>
}

impl<T: Read + Write> Plain<T> {
    pub fn new(stream: T) -> Plain<T> {
        Plain {
            inner: stream,
            rx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            tx_buf: Vec::<u8>::with_capacity(BUF_SIZE)
        }
    }
}

impl<T: Read + Write> Blocking for Plain<T> {
    fn b_recv(&mut self) -> Result<Vec<u8>, Error> {
        loop {
            let mut buf = [0u8; BUF_SIZE];
            let read_result = self.inner.read(&mut buf);
            if read_result.is_err() {
                let err = read_result.unwrap_err();
                return Err(err);
            }

            let num_read = read_result.unwrap();
            self.rx_buf.extend_from_slice(&buf[0..num_read]);

            match frame::from_raw_parts(&mut self.rx_buf) {
                Some(frame) => {
                    return Ok(frame);
                }
                None => { }
            };
        }
    }

    fn b_send(&mut self, buf: &[u8]) -> Result<(), Error> {
        let frame = frame::new(buf);
        let write_result = self.inner.write(&frame[..]);
        if write_result.is_err() {
            let err = write_result.unwrap_err();
            return Err(err);
        }

        Ok(())
    }
}

impl<T: Read + Write> NonBlocking for Plain<T> {
    fn nb_recv(&mut self) -> Result<Vec<Vec<u8>>, Error> {
        loop {
            let mut buf = [0u8; BUF_SIZE];
            let read_result = self.inner.read(&mut buf);
            if read_result.is_err() {
                let err = read_result.unwrap_err();
                if err.kind() == ErrorKind::WouldBlock {
                    trace!("Received WouldBlock");
                    break;
                }
                return Err(err);
            }

            let num_read = read_result.unwrap();
            self.rx_buf.extend_from_slice(&buf[0..num_read]);
        }

        let mut ret_buf = Vec::<Vec<u8>>::with_capacity(5);
        while let Some(frame) = frame::from_raw_parts(&mut self.rx_buf) {
            ret_buf.push(frame);
        }

        if ret_buf.len() > 0 {
            return Ok(ret_buf);
        }

        Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"))
    }

    fn nb_send(&mut self, buf: &[u8]) -> Result<(), Error> {
        let frame = frame::new(buf);
        self.tx_buf.extend_from_slice(&frame[..]);

        let write_result = self.inner.write(&self.tx_buf[..]);
        if write_result.is_err() {
            let err = write_result.unwrap_err();
            return Err(err);
        }

        let num_written = write_result.unwrap();
        if num_written == 0 {
            return Err(Error::new(ErrorKind::Other, "Write returned zero"));
        }

        if num_written < self.tx_buf.len() {
            let tx_buf_len = self.tx_buf.len();
            let remaining_len = self.tx_buf.len() - num_written;

            let mut buf = Vec::<u8>::with_capacity(remaining_len);
            buf.extend_from_slice(&self.tx_buf[num_written..tx_buf_len]);

            mem::swap(&mut buf, &mut self.tx_buf);

            return Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"));
        }

        Ok(())
    }
}

impl<T: Read + Write + AsRawFd> AsRawFd for Plain<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}
