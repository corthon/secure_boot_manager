// Copyright (c) 2019 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]

pub mod encoder;

#[macro_use]
mod macros;

mod os_str;
pub use os_str::OsStr;

#[cfg(feature = "string")]
mod os_string;
#[cfg(feature = "string")]
pub use os_string::OsString;

#[cfg(test)]
mod tests {
    use crate::OsStr;
    #[test]
    fn test_os_str() {
        let path = [0x4e2du16, 0x56fdu16, 0x0u16];
        let path_osstr = OsStr::from_char16_with_nul(&path[..] as *const [u16] as *const u16);
        println!("path is {}", path_osstr);
        let path_osstr_nul = OsStr::from_u16_slice(&path[..]);
        let path_osstr = OsStr::from_u16_slice_with_nul(&path[..]);
        println!("path is {}", path_osstr);
        assert_eq!(path_osstr, "中国");
        assert_eq!(path_osstr_nul, "中国\0");
        assert_ne!(path_osstr, path_osstr_nul);
        assert_ne!("中1", path_osstr);
    }
}
