// Copyright (c) 2019 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::fmt;
use core::slice::Iter;

pub struct OsStr([u16]);

#[cfg(feature = "string")]
use crate::os_string::OsString;

impl OsStr {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &OsStr {
        s.as_ref()
    }

    pub fn as_u16_slice(&self) -> &[u16] {
        &self.0[..]
    }

    #[cfg(feature = "string")]
    pub fn to_os_string(&self) -> OsString {
        let mut s = OsString::new();
        s.push(self);
        s
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn from_u16_slice_with_nul(s: &[u16]) -> &OsStr {
        unsafe {
            let len = OsStr::char16_with_null_len(s as *const [u16] as *const u16);
            &*(&s[0..len] as *const [u16] as *const OsStr)
        }
    }

    pub fn from_u16_slice(s: &[u16]) -> &OsStr {
        unsafe { &*(s as *const [u16] as *const OsStr) }
    }

    pub fn from_u16_slice_mut(s: &mut [u16]) -> &mut OsStr {
        unsafe { &mut *(s as *mut [u16] as *mut OsStr) }
    }

    unsafe fn char16_with_null_len(s: *const u16) -> usize {
        let mut len = 0;
        loop {
            let v = ((*(((s as u64) + (2 * len as u64)) as *const u16)) & 0xffu16) as u32;

            if v == 0 {
                break;
            }
            len += 1;
        }
        len
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn from_char16_with_nul(s: *const u16) -> &'static Self {
        let s = unsafe { core::slice::from_raw_parts(s, Self::char16_with_null_len(s)) };
        OsStr::from_u16_slice(s)
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn from_char16_with_nul_mut(s: *mut u16) -> &'static mut Self {
        let s = unsafe { core::slice::from_raw_parts_mut(s as *mut u16, Self::char16_with_null_len(s)) };
        OsStr::from_u16_slice_mut(s)
    }

    pub fn iter(&self) -> Iter<'_, u16> {
        self.0.iter()
    }

    fn format_fn(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        const BUFFER_LEN: usize = 42;

        let mut buffer = [0u8; BUFFER_LEN * 3 + 1];

        let mut end_index;
        let mut res: core::result::Result<(), core::fmt::Error> = Ok(());
        for i in 0..((len + BUFFER_LEN) / BUFFER_LEN) {
            if (i + 1) * BUFFER_LEN >= len {
                end_index = len;
            } else {
                end_index = (i + 1) * BUFFER_LEN;
            }
            let ret = crate::encoder::decode(&(self.0[i * BUFFER_LEN..end_index]), &mut buffer);
            if let Ok(length) = ret {
                res = write!(f, "{}", core::str::from_utf8(&buffer[..length]).expect("error encoder"));
                res?
            }
        }
        res
    }
}

impl fmt::Debug for &OsStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_fn(f)
    }
}

impl fmt::Debug for &mut OsStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_fn(f)
    }
}

impl fmt::Display for &OsStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_fn(f)
    }
}

impl fmt::Display for &mut OsStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_fn(f)
    }
}

impl AsRef<OsStr> for OsStr {
    fn as_ref(&self) -> &OsStr {
        self
    }
}

impl core::cmp::PartialEq<str> for OsStr {
    fn eq(&self, other: &str) -> bool {
        if self.0.len() == other.chars().count() {
            for (i, c) in other.chars().enumerate() {
                if c as u32 != self.0[i] as u32 {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

impl core::cmp::PartialEq<OsStr> for str {
    fn eq(&self, other: &OsStr) -> bool {
        if other.0.len() == self.chars().count() {
            for (i, c) in self.chars().enumerate() {
                if c as u32 != other.0[i] as u32 {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

impl core::cmp::PartialEq<OsStr> for OsStr {
    fn eq(&self, other: &OsStr) -> bool {
        if other.0.len() == self.0.len() {
            return self.0 == other.0;
        }
        false
    }
}
