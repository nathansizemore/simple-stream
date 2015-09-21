// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.


//! SimpleStream crate.


extern crate libc;
extern crate errno;


pub mod message;
pub mod readbuffer;
pub mod bstream;
pub mod nbetstream;
