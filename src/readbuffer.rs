// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible
// With Secondary Licenses", as defined by
// the Mozilla Public License, v. 2.0.


//! ReadBuffer crate.


use super::message::Message;

pub struct ReadBuffer {
    /// Current message
    c_msg: Message,
    /// Current bytes remaining for next read
    c_remaining: u16,
    /// Queue of messages created during last read
    queue: Vec<Message>
}


impl ReadBuffer {

    /// Creates a new ReadBuffer
    pub fn new() -> ReadBuffer {
        ReadBuffer {
            c_msg: Message::new(),
            c_remaining: 0u16,
            queue: Vec::<Message>::new()
        }
    }

    /// Returns whether or not the struct is in a state to start with a new Message
    pub fn frame_complete() -> bool {
        if c_msg.len == c_msg.payload.len() {
            true
        }
        false
    }
}
