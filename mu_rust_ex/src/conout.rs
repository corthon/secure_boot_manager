// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::cell::RefCell;
use core::ptr::NonNull;
use r_efi::efi;
use string::OsString;

use crate::UefiResult;

pub struct ConOut {
    inner: NonNull<efi::protocols::simple_text_output::Protocol>,
}

// NOTE: Could even wrap this in a RefCell if we wanted to control re-entrance.
static mut CON_OUT: Option<RefCell<ConOut>> = None;

impl ConOut {
    fn new(st_ptr: *mut efi::SystemTable) -> UefiResult<Self> {
        let st = unsafe { st_ptr.as_ref() }.ok_or(efi::Status::INVALID_PARAMETER)?;
        NonNull::new(st.con_out)
            .map(|inner| Self { inner })
            .ok_or(efi::Status::INVALID_PARAMETER)
    }

    pub unsafe fn init(st_ptr: *mut efi::SystemTable) -> UefiResult<()> {
        let co = Self::new(st_ptr)?;
        CON_OUT = Some(RefCell::new(co));
        Ok(())
    }

    pub fn print(&mut self, out_string: &str) {
        let co = self.inner.as_ptr();
        unsafe {
            ((*co).output_string)(
                co,
                OsString::from(out_string).as_mut_ptr() as *mut efi::Char16,
            );
        }
    }
}

impl core::fmt::Write for ConOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    match unsafe { CON_OUT.as_mut() } {
        Some(co) => {
            co.borrow_mut()
                .write_fmt(args)
                .expect("error in ConOut write");
        }
        _ => (),
    }
}

#[macro_export]
macro_rules! print {
    // ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
    ($($arg:tt)*) => ($crate::conout::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}
