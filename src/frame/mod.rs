// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


pub mod simple;
pub mod websocket;


pub trait Frame: Sync + Send {
    fn to_bytes(&self) -> Vec<u8>;
    fn payload(&self) -> Vec<u8>;
    fn len_as_vec(&self) -> usize;
    fn as_mut_raw_erased(&self) -> *mut ();
}

pub trait FrameBuilder {
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<Frame>>;
}
