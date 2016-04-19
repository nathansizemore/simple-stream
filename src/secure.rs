// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::mem;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use openssl::ssl::SslStream;
use openssl::ssl::error::Error as SslStreamError;

use frame;
use super::{Blocking, NonBlocking};


const BUF_SIZE: usize = 1024;


pub struct Secure<T: Read + Write> {
    inner: SslStream<T>,
    rx_buf: Vec<u8>,
    tx_buf: Vec<u8>
}


impl<T: Read + Write> Secure<T> {
    pub fn new(stream: SslStream<T>) -> Secure<T> {
        Secure {
            inner: stream,
            rx_buf: Vec::<u8>::with_capacity(BUF_SIZE),
            tx_buf: Vec::<u8>::with_capacity(BUF_SIZE)
        }
    }
}

impl<T: Read + Write> Blocking for Secure<T> {
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

impl<T: Read + Write> NonBlocking for Secure<T> {
    fn nb_recv(&mut self) -> Result<Vec<Vec<u8>>, Error> {
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

        let write_result = self.inner.ssl_write(&self.tx_buf[..]);
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

impl<T: Read + Write + AsRawFd> AsRawFd for Secure<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}
