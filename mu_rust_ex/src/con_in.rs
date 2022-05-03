// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::ptr::NonNull;
use r_efi::{efi, protocols::simple_text_input::InputKey};

use crate::boot::uefi_bs;
use crate::{boot::BootServices, println, UefiResult};

pub struct ConIn(NonNull<efi::protocols::simple_text_input::Protocol>);
impl ConIn {
    // [unsafe] Caller must ensure that `st` is a valid, aligned pointer to
    //          the UEFI SystemTable.
    pub unsafe fn new(st: *mut efi::SystemTable) -> UefiResult<Self> {
        Ok(Self(
            NonNull::new((*st).con_in).ok_or(efi::Status::INVALID_PARAMETER)?,
        ))
    }

    pub fn get_char(&self) -> UefiResult<char> {
        // [unsafe] See Self::new()
        let prot = unsafe { self.0.as_ptr().as_mut().unwrap() };

        // Wait for the key event.
        let bs = uefi_bs();
        let key_event = [prot.wait_for_key];
        bs.wait_for_event(&key_event)?;

        let mut key: InputKey = InputKey {
            ..Default::default()
        };
        (prot.read_key_stroke)(prot as *mut _, &mut key as *mut _);
        println!("Key {:?}", key);
        Ok('J')
    }
}
