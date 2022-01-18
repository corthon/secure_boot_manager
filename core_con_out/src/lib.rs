// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![no_std]

use core::cell::RefCell;
use core::ptr::NonNull;
use r_efi::efi;
use spin::Mutex;

type UefiResult<T> = Result<T, efi::Status>;

pub struct ConOut {
    inner: NonNull<efi::protocols::simple_text_output::Protocol>,
}

const CON_OUT_BUFFER_SIZE: usize = 0x400;

static mut CON_OUT: Option<RefCell<ConOut>> = None;
static mut CON_OUT_BUFFER: Mutex<[u16; CON_OUT_BUFFER_SIZE]> =
    Mutex::new([0u16; CON_OUT_BUFFER_SIZE]);

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
        let mut buffer = unsafe { CON_OUT_BUFFER.lock() };

        let mut i: usize = 0;
        for utf8byte in out_string.bytes() {
            buffer[i] = match utf8byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\r' => utf8byte as u16,
                // not part of printable ASCII range
                _ => 0xfe as u16,
            };
            i += 1;
            if i > (CON_OUT_BUFFER_SIZE - 2) {
                break;
            }
        }
        buffer[i] = 0u16;

        unsafe {
            ((*co).output_string)(co, buffer.as_mut_ptr() as *mut efi::Char16);
        }
    }
}

impl core::fmt::Write for ConOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

// This function is especially unsafe in that it will seize the RefCell regardless
// of whether it is currently borrowed. Only to be used in panic situations
// when there is no expectation to return.
pub unsafe fn get_conout_panic() -> Option<&'static mut ConOut> {
    CON_OUT.as_mut().map(|rc| rc.get_mut())
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
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}
