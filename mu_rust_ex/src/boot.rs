// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

extern crate alloc;

use alloc::vec::Vec;

use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use r_efi::efi;

use efi::BootServices as EfiBootServices;

use crate::UefiResult;

pub struct BootServices {
    inner: NonNull<EfiBootServices>,
}

// NOTE: This is probably not actually thread safe, and should instead use a proper Mutex,
//       but this will do. Also, this may not be necessary.
static mut BOOT_SERVICES: Option<RefCell<BootServices>> = None;
pub unsafe fn init_uefi_bs(st_ptr: *mut efi::SystemTable) -> UefiResult<()> {
    let bs = BootServices::new(st_ptr)?;
    BOOT_SERVICES = Some(RefCell::new(bs));
    Ok(())
}
pub fn uefi_bs() -> impl Deref<Target = BootServices> {
    // We're just going to panic if any of this fails.
    unsafe {
        BOOT_SERVICES
            .as_ref()
            .map(|opt_bs| opt_bs.borrow())
            .unwrap()
    }
}
pub fn uefi_bs_mut() -> impl DerefMut<Target = BootServices> {
    // We're just going to panic if any of this fails.
    unsafe {
        BOOT_SERVICES
            .as_ref()
            .map(|opt_bs| opt_bs.borrow_mut())
            .unwrap()
    }
}

impl BootServices {
    pub fn new(st_ptr: *mut efi::SystemTable) -> UefiResult<Self> {
        let st = unsafe { st_ptr.as_ref() }.ok_or(efi::Status::INVALID_PARAMETER)?;
        Ok(Self {
            inner: NonNull::new(st.boot_services).ok_or(efi::Status::INVALID_PARAMETER)?,
        })
    }

    pub fn locate_protocol(&self, protocol: &efi::Guid) -> UefiResult<*mut core::ffi::c_void> {
        let bs = unsafe { self.inner.as_ref() };
        let mut inner_guid = *protocol;
        let mut ret_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let status = (bs.locate_protocol)(
            &mut inner_guid as *mut _,
            core::ptr::null_mut(),
            &mut ret_ptr as *mut _,
        );

        if !status.is_error() {
            Ok(ret_ptr)
        } else {
            Err(status)
        }
    }

    pub fn locate_protocol_handles(&self, protocol: &efi::Guid) -> UefiResult<Vec<efi::Handle>> {
        let bs = unsafe { self.inner.as_ref() };
        let mut inner_guid = *protocol;
        let mut buffer_size: usize = 0;

        let status = (bs.locate_handle)(
            efi::BY_PROTOCOL,
            &mut inner_guid as *mut _,
            core::ptr::null_mut(),
            &mut buffer_size as *mut _,
            core::ptr::null_mut(),
        );

        if status.is_error() && status != efi::Status::BUFFER_TOO_SMALL {
            return Err(status);
        }

        let buffer_count = buffer_size / core::mem::size_of::<efi::Handle>();
        let mut buffer = Vec::<efi::Handle>::with_capacity(buffer_count);

        let status = (bs.locate_handle)(
            efi::BY_PROTOCOL,
            &mut inner_guid as *mut _,
            core::ptr::null_mut(),
            &mut buffer_size as *mut _,
            buffer.as_mut_ptr() as *mut _,
        );

        if !status.is_error() {
            unsafe { buffer.set_len(buffer_count) };
            Ok(buffer)
        } else {
            Err(status)
        }
    }

    pub fn get_protocol(
        &self,
        protocol: &efi::Guid,
        handle: efi::Handle,
    ) -> UefiResult<*mut core::ffi::c_void> {
        let bs = unsafe { self.inner.as_ref() };
        let mut inner_guid = *protocol;
        let mut prot_ptr: *mut core::ffi::c_void = core::ptr::null_mut();

        let status = (bs.open_protocol)(
            handle,
            &mut inner_guid as *mut _,
            &mut prot_ptr as *mut _,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            efi::OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
        );

        if !status.is_error() {
            Ok(prot_ptr)
        } else {
            Err(status)
        }
    }
}
