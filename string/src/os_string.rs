// Copyright (c) 2019 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

extern crate alloc;
pub use alloc::vec::Vec;
use core::fmt;

use crate::OsStr;

pub struct OsString(Vec<u16>);

impl OsString {
    pub fn new() -> OsString {
        OsString(Vec::new())
    }

    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.0.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }

    pub fn push<T: AsRef<OsStr>>(&mut self, s: T) {
        for v in s.as_ref().iter() {
            self.0.push(*v)
        }
    }
}

impl Default for OsString {
    fn default() -> OsString {
        OsString::new()
    }
}

impl From<&str> for OsString {
    // Get OsString object from &str
    // if error occur, immediately return.
    fn from(s: &str) -> OsString {
        let mut res = OsString::new();

        let add_char = |ret| match ret {
            Ok(ch) => {
                res.0.push(ch);
                Ok(res.0.len())
            }
            Err(err) => Err(err),
        };

        crate::encoder::encode_fnc(s, add_char).unwrap_or(0);
        res.0.push(0u16);
        res
    }
}

impl From<&OsStr> for OsString {
    // Get OsString object from &str
    // if error occur, immediately return.
    fn from(s: &OsStr) -> OsString {
        let mut res = OsString::new();
        res.push(s);
        res
    }
}

impl fmt::Display for OsString {
    // TODO: directly output u16
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        let mut vec: Vec<u8> = Vec::new();
        vec.resize(len * 3 + 1, 0u8);
        let _res = crate::encoder::decode(&(self.0), vec.as_mut_slice());
        write!(f, "{}", core::str::from_utf8(&vec[..]).unwrap())
    }
}

impl core::ops::Deref for OsString {
    type Target = OsStr;

    fn deref(&self) -> &OsStr {
        &self[..]
    }
}

impl core::ops::Index<core::ops::RangeFull> for OsString {
    type Output = OsStr;
    fn index(&self, _index: core::ops::RangeFull) -> &OsStr {
        OsStr::from_u16_slice(&(self.0[..]))
    }
}

impl AsRef<OsStr> for OsString {
    fn as_ref(&self) -> &OsStr {
        self
    }
}
