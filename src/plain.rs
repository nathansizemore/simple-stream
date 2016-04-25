// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::mem;
use std::any::Any;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use libc;
use errno::errno;

use frame::Frame;
use super::{Blocking, NonBlocking};


const BUF_SIZE: usize = 1024;


#[derive(Clone)]
pub struct Plain<S, F> where
    S: Read + Write,
    F: Frame
{
    inner: S,
    frame: F,
    rx_buf: Vec<u8>,
    tx_buf: Vec<u8>
}

impl<S, F> Plain<S, F> where
    S: Read + Write,
    F: Frame
{
    pub fn new(stream: S, frame: F) -> Plain<S, F> {
        Plain {
            inner: stream,
            frame: frame,
            rx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            tx_buf: Vec::<u8>::with_capacity(BUF_SIZE)
        }
    }
}

impl<S, F> Blocking for Plain<S, F> where
    S: Read + Write,
    F: Frame
{
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

            match F::from_bytes(&mut self.rx_buf) {
                Some(boxed_frame) => {
                    return Ok(boxed_frame.payload());
                }
                None => { }
            };
        }
    }

    fn b_send<T: Any>(&mut self, buf: &[u8], args: &Vec<T>) -> Result<(), Error> {
        let frame = F::new(buf, args);
        let out_buf = frame.to_bytes();
        let write_result = self.inner.write(&out_buf[..]);
        if write_result.is_err() {
            let err = write_result.unwrap_err();
            return Err(err);
        }

        Ok(())
    }
}

impl<S, F> NonBlocking for Plain<S, F> where
    S: Read + Write,
    F: Frame
{
    fn nb_recv(&mut self) -> Result<Vec<Vec<u8>>, Error> {
        loop {
            let mut buf = [0u8; BUF_SIZE];
            let read_result = self.inner.read(&mut buf);
            if read_result.is_err() {
                let err = read_result.unwrap_err();
                if err.kind() == ErrorKind::WouldBlock {
                    break;
                }
                return Err(err);
            }

            let num_read = read_result.unwrap();
            self.rx_buf.extend_from_slice(&buf[0..num_read]);
        }

        let mut ret_buf = Vec::<Vec<u8>>::with_capacity(5);
        while let Some(boxed_frame) = F::from_bytes(&mut self.rx_buf) {
            ret_buf.push(boxed_frame.payload());
        }

        if ret_buf.len() > 0 {
            return Ok(ret_buf);
        }

        Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"))
    }

    fn nb_send<T: Any>(&mut self, buf: &[u8], args: &Vec<T>) -> Result<(), Error> {
        let frame = F::new(buf, args);
        self.tx_buf.extend_from_slice(&frame.to_bytes()[..]);

        let mut out_buf = Vec::<u8>::with_capacity(BUF_SIZE);
        mem::swap(&mut self.tx_buf, &mut out_buf);

        let write_result = self.inner.write(&out_buf[..]);
        if write_result.is_err() {
            let err = write_result.unwrap_err();
            return Err(err);
        }

        let num_written = write_result.unwrap();
        if num_written == 0 {
            return Err(Error::new(ErrorKind::Other, "Write returned zero"));
        }

        if num_written < out_buf.len() {
            let out_buf_len = out_buf.len();
            self.tx_buf.extend_from_slice(&out_buf[num_written..out_buf_len]);

            return Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"));
        }

        Ok(())
    }
}

impl<S, F> AsRawFd for Plain<S, F> where
    S: Read + Write + AsRawFd,
    F: Frame
{
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl<S, F> Plain<S, F> where
    S: Read + Write + AsRawFd,
    F: Frame
{
    pub fn shutdown(&mut self) -> Result<(), Error> {
        let result = unsafe {
            libc::shutdown(self.as_raw_fd(), libc::SHUT_RDWR)
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Error> {
        let result = unsafe {
            libc::close(self.as_raw_fd())
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}
