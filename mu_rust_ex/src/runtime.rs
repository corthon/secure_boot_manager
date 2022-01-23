// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use r_efi::efi;

use efi::RuntimeServices as EfiRuntimeServices;

use crate::UefiResult;

#[derive(Debug, Clone, Copy)]
pub enum RuntimeServicesError {
    BadSize(usize, efi::Status),
    Other(efi::Status),
}
impl core::fmt::Display for RuntimeServicesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UEFI RuntimeServices Error")
    }
}
impl From<RuntimeServicesError> for efi::Status {
    fn from(f: RuntimeServicesError) -> Self {
        match f {
            RuntimeServicesError::Other(x) => x,
            RuntimeServicesError::BadSize(_, x) => x,
        }
    }
}

pub type UefiRsResult<T> = Result<T, RuntimeServicesError>;

pub struct RuntimeServices {
    inner: NonNull<EfiRuntimeServices>,
}

// NOTE: This is probably not actually thread safe, and should instead use a proper Mutex,
//       but this will do. Also, this may not be necessary.
static mut RUNTIME_SERVICES: Option<RefCell<RuntimeServices>> = None;
pub unsafe fn init_uefi_rs(st_ptr: *mut efi::SystemTable) -> UefiResult<()> {
    let bs = RuntimeServices::new(st_ptr)?;
    RUNTIME_SERVICES = Some(RefCell::new(bs));
    Ok(())
}
pub fn uefi_rs() -> impl Deref<Target = RuntimeServices> {
    // We're just going to panic if any of this fails.
    unsafe {
        RUNTIME_SERVICES
            .as_ref()
            .map(|opt_bs| opt_bs.borrow())
            .unwrap()
    }
}
pub fn uefi_rs_mut() -> impl DerefMut<Target = RuntimeServices> {
    // We're just going to panic if any of this fails.
    unsafe {
        RUNTIME_SERVICES
            .as_ref()
            .map(|opt_bs| opt_bs.borrow_mut())
            .unwrap()
    }
}

impl RuntimeServices {
    pub fn new(st_ptr: *mut efi::SystemTable) -> UefiResult<Self> {
        let st = unsafe { st_ptr.as_ref() }.ok_or(efi::Status::INVALID_PARAMETER)?;
        Ok(Self {
            inner: NonNull::new(st.runtime_services).ok_or(efi::Status::INVALID_PARAMETER)?,
        })
    }

    pub fn get_variable_size(
        &self,
        name: *mut efi::Char16,
        guid: *mut efi::Guid,
    ) -> UefiResult<usize> {
        let rs = unsafe { self.inner.as_ref() };
        let mut data_size: usize = 0;
        let status = (rs.get_variable)(
            name,
            guid,
            core::ptr::null_mut(),
            &mut data_size as *mut _,
            core::ptr::null_mut(),
        );

        if !status.is_error() {
            Ok(data_size)
        } else {
            Err(status)
        }
    }

    pub fn get_variable(
        &self,
        name: *mut efi::Char16,
        guid: *mut efi::Guid,
        data: &mut [u8],
    ) -> UefiRsResult<(usize, u32)> {
        let rs = unsafe { self.inner.as_ref() };
        let mut attributes: u32 = 0;
        let mut data_size = data.len();

        let status = (rs.get_variable)(
            name,
            guid,
            &mut attributes as *mut _,
            &mut data_size as *mut _,
            data.as_mut_ptr() as *mut _,
        );

        if !status.is_error() {
            Ok((data_size, attributes))
        } else if [efi::Status::BAD_BUFFER_SIZE, efi::Status::BUFFER_TOO_SMALL].contains(&status) {
            Err(RuntimeServicesError::BadSize(data_size, status))
        } else {
            Err(RuntimeServicesError::Other(status))
        }
    }
}
