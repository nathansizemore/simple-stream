// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.




bitflags! {
    flags OpCode: u8 {
        const CONTINUATION  = 0b0000_0000,
        const TEXT          = 0b0000_0001,
        const BINARY        = 0b0000_0010,
        const CLOSE         = 0b0000_1000,
        const PING          = 0b0000_1001,
        const PONG          = 0b0000_1010
    }
}

enum FrameType {
    Control,
    Data
}

struct Header {
    op_code: u8,
    mask: bool,
    payload_len: u64,
    masking_key: [u8; 4]
}

struct Payload {
    data: Vec<u8>
}

struct Frame {
    header: Header,
    payload: Payload
}
