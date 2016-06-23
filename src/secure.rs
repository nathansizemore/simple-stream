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

use openssl::ssl::SslStream;
use openssl::ssl::error::Error as SslStreamError;

use frame::{Frame, FrameBuilder};
use super::{Blocking, NonBlocking};


const BUF_SIZE: usize = 1024;


/// OpenSSL backed stream.
#[derive(Clone)]
pub struct Secure<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    inner: SslStream<S>,
    rx_buf: Vec<u8>,
    tx_buf: Vec<u8>,
    phantom: PhantomData<FB>
}

impl<S, FB> Secure<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    /// Creates a new secured stream.
    pub fn new(stream: SslStream<S>) -> Secure<S, FB> {
        Secure {
            inner: stream,
            rx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            tx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            phantom: PhantomData
        }
    }
}

impl<S, FB> Blocking for Secure<S, FB> where
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

impl<S, FB> NonBlocking for Secure<S, FB> where
    S: Read + Write,
    FB: FrameBuilder
{
    fn nb_recv(&mut self) -> Result<Vec<Box<Frame>>, Error> {
        loop {
            let mut buf = [0u8; BUF_SIZE];
            let read_result = self.inner.ssl_read(&mut buf);
            if read_result.is_err() {
                let err = read_result.unwrap_err();
                match err {
                    SslStreamError::ZeroReturn => {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "UnexpectedEof"));
                    }
                    SslStreamError::WantRead(_) => {
                        break;
                    }
                    SslStreamError::WantX509Lookup => {
                        return Err(Error::new(ErrorKind::Other, "WantX509Lookup"));
                    }
                    SslStreamError::Stream(e) => {
                        return Err(e);
                    }
                    SslStreamError::Ssl(ssl_errs) => {
                        let mut err_str = String::new();
                        err_str.push_str("The following Ssl Error codes were thrown: ");

                        for ssl_err in ssl_errs.iter() {
                            err_str.push_str(&(format!("{} ", ssl_err.error_code())[..]));
                        }

                        return Err(Error::new(ErrorKind::Other, &err_str[..]));
                    }
                    _ => {
                        // Other error types should not be thrown from this operation
                        return Err(Error::new(ErrorKind::Other, "Unknown error during ssl_read"));
                    }
                };
            }

            let num_read = read_result.unwrap();
            trace!("Read {} byte(s)", num_read);
            self.rx_buf.extend_from_slice(&buf[0..num_read]);
        }

        let mut ret_buf = Vec::<Box<Frame>>::with_capacity(5);
        while let Some(boxed_frame) = FB::from_bytes(&mut self.rx_buf) {
            info!("Complete frame read");
            ret_buf.push(boxed_frame);
        }

        if ret_buf.len() > 0 {
            info!("Read {} frame(s)", ret_buf.len());
            return Ok(ret_buf);
        }

        Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"))
    }

    fn nb_send(&mut self, frame: &Frame) -> Result<(), Error> {
        self.tx_buf.extend_from_slice(&frame.to_bytes()[..]);

        let mut out_buf = Vec::<u8>::with_capacity(BUF_SIZE);
        mem::swap(&mut self.tx_buf, &mut out_buf);

        let write_result = self.inner.ssl_write(&out_buf[..]);
        if write_result.is_err() {
            let err = write_result.unwrap_err();
            match err {
                SslStreamError::WantWrite(_) => {
                    return Err(Error::new(ErrorKind::WouldBlock, "WouldBlock"));
                }
                SslStreamError::WantX509Lookup => {
                    return Err(Error::new(ErrorKind::Other, "WantX509Lookup"));
                }
                SslStreamError::Stream(e) => {
                    return Err(e);
                }
                SslStreamError::Ssl(ssl_errs) => {
                    let mut err_str = String::new();
                    err_str.push_str("The following Ssl Error codes were thrown: ");

                    for ssl_err in ssl_errs.iter() {
                        err_str.push_str(&(format!("{} ", ssl_err.error_code())[..]));
                    }

                    return Err(Error::new(ErrorKind::Other, &err_str[..]));
                }
                _ => {
                    // Other error types should not be thrown from this operation
                    return Err(Error::new(ErrorKind::Other, "Unknown error during ssl_write"))
                }
            };
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

impl<S, FB> AsRawFd for Secure<S, FB> where
    S: Read + Write + AsRawFd,
    FB: FrameBuilder
{
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}
