// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.



#[macro_use]
extern crate log;
extern crate libc;
extern crate errno;
extern crate openssl;
#[macro_use]
extern crate bitflags;

use std::io::Error;

pub use plain::*;
pub use socket::*;
pub use secure::*;

mod frame;
mod socket;
mod plain;
mod secure;



pub trait Blocking {
    fn b_recv(&mut self) -> Result<Vec<u8>, Error>;
    fn b_send(&mut self, buf: &[u8]) -> Result<(), Error>;
}

pub trait NonBlocking {
    fn nb_recv(&mut self) -> Result<Vec<Vec<u8>>, Error>;
    fn nb_send(&mut self, buf: &[u8]) -> Result<(), Error>;
}

*  %x0 denotes a continuation frame

      *  %x1 denotes a text frame

      *  %x2 denotes a binary frame

      *  %x3-7 are reserved for further non-control frames

      *  %x8 denotes a connection close

      *  %x9 denotes a ping

      *  %xA denotes a pong
