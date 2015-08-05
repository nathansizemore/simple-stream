// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.


//! Message crate.

#[derive(Clone)]
pub struct Message {
    pub len: u16,
    pub payload: Vec<u8>
}


impl Message {

    /// Creates a new message
    pub fn new() -> Message {
        Message {
            len: 0,
            payload: Vec::<u8>::new()
        }
    }
}
