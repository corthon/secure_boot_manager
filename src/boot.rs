// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::ptr::NonNull;
use r_efi::efi;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use efi::BootServices as EfiBootServices;

pub struct BootServices {
    pub inner: NonNull<EfiBootServices>,
}

impl BootServices {
    pub fn new(st_ptr: *mut efi::SystemTable) -> Self {
        unsafe {
            Self {
                inner: NonNull::new((*st_ptr).boot_services).unwrap(),
            }
        }
    }
}
