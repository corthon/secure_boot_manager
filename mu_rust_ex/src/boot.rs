// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::mem;
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
    unsafe fn new(st_ptr: *mut efi::SystemTable) -> UefiResult<Self> {
        let bs = (*st_ptr).boot_services;
        match NonNull::new(bs) {
            Some(nn) => Ok(Self { inner: nn }),
            _ => Err(efi::Status::INVALID_PARAMETER),
        }
    }

    pub unsafe fn init(st_ptr: *mut efi::SystemTable) -> UefiResult<()> {
        let bs = Self::new(st_ptr)?;
        BOOT_SERVICES = Some(bs);
        Ok(())
    }

    pub fn locate_protocol_handles(
        protocol: &mut efi::Guid,
        buffer: &mut [efi::Handle],
    ) -> UefiResult<usize> {
        let opt_bs = unsafe { &BOOT_SERVICES };
        if let Some(bs) = opt_bs {
            let mut buffer_size: usize = buffer.len() * mem::size_of::<efi::Handle>();

            let status = unsafe {
                ((*bs.inner.as_ptr()).locate_handle)(
                    efi::BY_PROTOCOL,
                    protocol as *mut _,
                    core::ptr::null_mut(),
                    &mut buffer_size as *mut _,
                    buffer.as_mut_ptr(),
                )
            };

            if !status.is_error() {
                Ok(buffer_size)
            } else {
                Err(status)
            }
        } else {
            Err(efi::Status::NOT_STARTED)
        }
    }
}
