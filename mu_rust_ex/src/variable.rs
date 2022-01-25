// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use r_efi::efi;
use string::OsString;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{runtime, UefiResult};

pub const EFI_GLOBAL_VARIABLE_GUID: efi::Guid = efi::Guid::from_fields(
    0x8BE4DF61,
    0x93CA,
    0x11D2,
    0xAA,
    0x0D,
    &[0x00, 0xE0, 0x98, 0x03, 0x2B, 0x8C],
);

#[derive(Debug, Clone)]
pub struct EfiVariable {
    pub name: String,
    pub guid: efi::Guid,
    pub data: Vec<u8>,
    pub attributes: u32,
}

impl EfiVariable {
    pub fn get_variable(name: &str, guid: &efi::Guid) -> UefiResult<Self> {
        let rs = runtime::uefi_rs();

        let mut name_string = OsString::from(name);
        let name_ptr = name_string.as_mut_ptr() as *mut efi::Char16;
        let mut local_guid = *guid;

        let data_size: usize = rs.get_variable_size(name_ptr, &mut local_guid as *mut _)?;

        let mut data = Vec::<u8>::with_capacity(data_size);
        let (data_size, attributes) =
            rs.get_variable(name_ptr, &mut local_guid as *mut _, &mut data)?;
        unsafe { data.set_len(data_size) };

        Ok(Self {
            name: String::from(name),
            guid: local_guid,
            data,
            attributes,
        })
    }
}
