// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::mem;
use std::marker::PhantomData;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use libc;
use errno::errno;

use frame::{Frame, FrameBuilder};
use super::{Blocking, NonBlocking};


const BUF_SIZE: usize = 1024;


/// Plain text stream.
#[derive(Clone)]
pub struct Plain<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    inner: S,
    rx_buf: Vec<u8>,
    tx_buf: Vec<u8>,
    phantom: PhantomData<FB>
}

impl<S, FB> Plain<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    /// Creates a new plain text stream.
    pub fn new(stream: S) -> Plain<S, FB> {
        Plain {
            inner: stream,
            rx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            tx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            phantom: PhantomData
        }
    }
}

impl<S, FB> Blocking for Plain<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    fn b_recv(&mut self) -> Result<Box<Frame>, Error> {
        // Empty anything that is in our buffer already from any previous reads
        match FB::from_bytes(&mut self.rx_buf) {
            Some(boxed_frame) => {
                debug!("Complete frame read");
                return Ok(boxed_frame);
            }
            None => { }
        };

        loop {
            let mut buf = [0u8; BUF_SIZE];
            let read_result = self.inner.read(&mut buf);
            if read_result.is_err() {
                let err = read_result.unwrap_err();
                return Err(err);
            }

            let num_read = read_result.unwrap();
            trace!("Read {} byte(s)", num_read);
            self.rx_buf.extend_from_slice(&buf[0..num_read]);

            match FB::from_bytes(&mut self.rx_buf) {
                Some(boxed_frame) => {
                    debug!("Complete frame read");
                    return Ok(boxed_frame);
                }
                None => { }
            };
        }
    }

    fn b_send(&mut self, frame: &Frame) -> Result<(), Error> {
        let out_buf = frame.to_bytes();
        let write_result = self.inner.write(&out_buf[..]);
        if write_result.is_err() {
            let err = write_result.unwrap_err();
            return Err(err);
        }

        trace!("Wrote {} byte(s)", write_result.unwrap());

        Ok(())
    }
}

impl<S, FB> NonBlocking for Plain<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    fn nb_recv(&mut self) -> Result<Vec<Box<Frame>>, Error> {
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
            trace!("Read {} byte(s)", num_read);
            self.rx_buf.extend_from_slice(&buf[0..num_read]);
        }

        let mut ret_buf = Vec::<Box<Frame>>::with_capacity(5);
        while let Some(boxed_frame) = FB::from_bytes(&mut self.rx_buf) {
            debug!("Complete frame read");
            ret_buf.push(boxed_frame);
        }

        if ret_buf.len() > 0 {
            debug!("Read {} frame(s)", ret_buf.len());
            return Ok(ret_buf);
        }

        Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"))
    }

    fn nb_send(&mut self, frame: &Frame) -> Result<(), Error> {
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

        trace!("Tried to write {} byte(s) wrote {} byte(s)", out_buf.len(), num_written);

        if num_written < out_buf.len() {
            let out_buf_len = out_buf.len();
            self.tx_buf.extend_from_slice(&out_buf[num_written..out_buf_len]);

            return Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"));
        }

        Ok(())
    }
}

impl<S, FB> AsRawFd for Plain<S, FB> where
    S: Read + Write + AsRawFd,
    FB: FrameBuilder
{
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl<S, FB> Plain<S, FB> where
    S: Read + Write + AsRawFd,
    FB: FrameBuilder
{
    /// Calls `libc::shutdown` on the underlying `RawFd`
    pub fn shutdown(&mut self) -> Result<(), Error> {
        trace!("Shutting down stream");
        let result = unsafe {
            libc::shutdown(self.as_raw_fd(), libc::SHUT_RDWR)
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    /// Calls `lib::close` on the underlying `RawFd`
    pub fn close(&mut self) -> Result<(), Error> {
        trace!("Closing stream");
        let result = unsafe {
            libc::close(self.as_raw_fd())
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}
