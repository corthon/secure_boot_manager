// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::convert::TryFrom;
use core::ptr::NonNull;
use r_efi::{efi, protocols::simple_text_input::InputKey as EfiInputKey};

use crate::boot::uefi_bs;
use crate::{println, UefiResult};

// #[repr(u8)]
// pub enum CtrlKey {
//     Null = 0,
//     CurUp,
//     CurDown,
//     CurRight,
//     CurLeft,
//     Home,
//     End,
//     Insert,
//     Del,
//     PgUp,
//     PgDown,
//     F1,
//     F2,
//     F3,
//     F4,
//     F5,
//     F6,
//     F7,
//     F8,
//     F9,
//     F10,
//     Esc,
// }
// impl TryFrom<u16> for CtrlKey {
//     type Error = ();
//     fn try_from(tf: u16) -> Result<Self, Self::Error> {
//         if tf <= CtrlKey::Esc as u16 {
//             Ok(tf as Self)
//         } else {
//             Err(())
//         }
//     }
// }
pub enum InputKey {
    Char(char),
    // Ctrl(CtrlKey),
    Ctrl(u16),
}

pub struct ConIn(NonNull<efi::protocols::simple_text_input::Protocol>);
impl ConIn {
    // [unsafe] Caller must ensure that `st` is a valid, aligned pointer to
    //          the UEFI SystemTable.
    pub unsafe fn new(st: *mut efi::SystemTable) -> UefiResult<Self> {
        Ok(Self(
            NonNull::new((*st).con_in).ok_or(efi::Status::INVALID_PARAMETER)?,
        ))
    }

    pub fn get_char(&self) -> UefiResult<InputKey> {
        // [unsafe] See Self::new()
        let prot = unsafe { self.0.as_ptr().as_mut().unwrap() };

        // Wait for the key event.
        let bs = uefi_bs();
        let key_event = [prot.wait_for_key];
        bs.wait_for_event(&key_event)?;

        let mut key: EfiInputKey = EfiInputKey {
            ..Default::default()
        };
        let status = (prot.read_key_stroke)(prot as *mut _, &mut key as *mut _);
        // println!("Keystroke {:?}", key);

        if !status.is_error() {
            match key.unicode_char {
                0 => Ok(InputKey::Ctrl(key.scan_code)),
                _ => Ok(InputKey::Char(
                    char::from_u32(key.unicode_char.into()).ok_or(efi::Status::NO_MAPPING)?,
                )),
            }
        } else {
            Err(status)
        }
    }
}
