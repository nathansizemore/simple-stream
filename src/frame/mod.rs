// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::any::Any;


pub mod simple;
pub mod websocket;


pub trait Frame {
    fn new<T: Any>(buf: &[u8], args: &Vec<T>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(buf: &mut Vec<u8>) -> Option<Box<Self>>;
    fn len_as_vec(&self) -> usize;
}
