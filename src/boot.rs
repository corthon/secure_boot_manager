// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::ptr::NonNull;
use r_efi::efi;

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

    pub fn open_protocol(&self, name: &str, guid: &efi::Guid) -> Result<EfiVariable, efi::Status> {
        let mut name_string = OsString::from(name);
        let mut local_guid = *guid;
        let mut attributes: u32 = 0;
        let mut data_size: usize = 0;

        // Get the size for the data.
        let mut status: efi::Status;
        unsafe {
            status = (self.inner.as_ref().get_variable)(
                name_string.as_mut_ptr() as *mut efi::Char16,
                &mut local_guid as *mut efi::Guid,
                &mut attributes as *mut u32,
                &mut data_size as *mut usize,
                core::ptr::null_mut(),
            );
        }
        if status != efi::Status::BUFFER_TOO_SMALL {
            return Err(status);
        }

        // Now that we've got the size, set up the Vector for data.
        let mut data = Vec::<u8>::with_capacity(data_size);
        unsafe {
            status = (self.inner.as_ref().get_variable)(
                name_string.as_mut_ptr() as *mut efi::Char16,
                &mut local_guid as *mut efi::Guid,
                &mut attributes as *mut u32,
                &mut data_size as *mut usize,
                data.as_mut_ptr() as *mut core::ffi::c_void,
            );
            if !status.is_error() {
                data.set_len(data_size);
            }
        }

        if status.is_error() {
            Err(status)
        } else {
            Ok(EfiVariable {
                name: String::from(name),
                guid: local_guid,
                data,
                attributes,
            })
        }
    }
}
