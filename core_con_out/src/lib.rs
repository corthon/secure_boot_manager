// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![no_std]

use core::fmt::Write;
use core::ptr::NonNull;
use r_efi::efi;
use spin::Mutex;

type UefiResult<T> = Result<T, efi::Status>;

const CON_OUT_BUFFER_SIZE: usize = 0x400;
pub struct ConOut {
    inner: Option<NonNull<efi::protocols::simple_text_output::Protocol>>,
    buffer: [u16; CON_OUT_BUFFER_SIZE],
}
// NOTE: UEFI isn't thread safe anyway, so we're not introducing too many problems
//       by allowing the protocol to be shared.
unsafe impl Sync for ConOut {}
unsafe impl Send for ConOut {}

// TODO: Move the buffer into the struct and move the Mutex around the option.
//       Before doing this, make sure you can still get to the pointer in a panic.
//       Looks like mutex.force_unlock would get it done.
static CON_OUT: Mutex<ConOut> = Mutex::new(ConOut {
    inner: None,
    buffer: [0u16; CON_OUT_BUFFER_SIZE],
});


impl ConOut {
    pub unsafe fn init(st_ptr: *mut efi::SystemTable) -> UefiResult<()> {
        let st = st_ptr.as_ref().ok_or(efi::Status::INVALID_PARAMETER)?;
        let mut co_guard = CON_OUT.lock();
        if co_guard.inner.is_some() {
            Err(efi::Status::ALREADY_STARTED)
        } else {
            co_guard.inner = NonNull::new(st.con_out);
            Ok(())
        }
    }

    pub fn print(&mut self, out_string: &str) {
        if let Some(ref mut co) = self.inner {
            let co = unsafe { co.as_mut() };
            let mut i: usize = 0;
            for utf8byte in out_string.bytes() {
                self.buffer[i] = match utf8byte {
                    // printable ASCII byte or newline
                    0x20..=0x7e | b'\n' | b'\r' => utf8byte as u16,
                    // not part of printable ASCII range
                    _ => 0xfe as u16,
                };
                i += 1;
                if i > (self.buffer.len() - 2) {
                    break;
                }
            }
            self.buffer[i] = 0u16;

            (co.output_string)(co as *mut _, self.buffer.as_mut_ptr() as *mut _);
        }
    }
}

impl core::fmt::Write for ConOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

// This function is especially unsafe in that it will seize the Mutex regardless
// of whether it is currently locked. Only to be used in panic situations
// when there is no expectation to return.
pub unsafe fn print_panic(args: ::core::fmt::Arguments) {
    CON_OUT.force_unlock();
    _ = CON_OUT.lock().write_fmt(args);
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    CON_OUT.lock().write_fmt(args).expect("error in ConOut write");
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
