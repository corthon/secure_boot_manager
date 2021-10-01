// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::ptr::NonNull;
use r_efi::efi;

use efi::BootServices as EfiBootServices;

use crate::UefiResult;

pub struct BootServices {
    inner: NonNull<EfiBootServices>,
}

// NOTE: Could even wrap this in a RefCell if we wanted to control re-entrance.
static mut BOOT_SERVICES: Option<BootServices> = None;

impl BootServices {
    fn new(st_ptr: *mut efi::SystemTable) -> UefiResult<Self> {
        let bs = unsafe { (*st_ptr).boot_services };
        match NonNull::new(bs) {
            Some(nn) => Ok(Self { inner: nn }),
            _ => Err(efi::Status::INVALID_PARAMETER)
        }
    }
    
    pub fn init(st_ptr: *mut efi::SystemTable) -> UefiResult<()> {
        let bs = Self::new(st_ptr)?;
        unsafe { BOOT_SERVICES = Some(bs) };
        Ok(())
    }
}
