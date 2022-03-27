// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::protocol_utility::{
    ManagedProtocol, RustProtocol, RustProtocolError as RPError, RustProtocolResult as RPResult,
};
use crate::{shell_protocol, UefiResult};

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use r_efi::efi;
use r_efi_string::str16::EfiStr16;
use spin::Mutex;

pub const PROTOCOL_NAME: &str = "EfiShellParametersProtocol";
pub const PROTOCOL_GUID: efi::Guid = efi::Guid::from_fields(
    0x752f3136,
    0x4e16,
    0x4fdc,
    0xa2,
    0x2a,
    &[0xe5, 0xf4, 0x68, 0x12, 0xf4, 0xca],
);

#[repr(C)]
pub struct RawProtocol {
    pub argv: *const *const efi::Char16,
    pub argc: usize,
    pub std_in: shell_protocol::FileHandle,
    pub std_out: shell_protocol::FileHandle,
    pub std_err: shell_protocol::FileHandle,
}

pub struct Protocol {
    inner: Arc<Mutex<Option<ManagedProtocol<RawProtocol>>>>,
}

impl Protocol {
    // NOTE: It's important that these all return ManagedProtocolError::Unregistered
    //       if the Option has been taken.
    pub fn get_args(&self) -> RPResult<Vec<String>> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;

        let result: Vec<String> = (0..prot.argc)
            .map(|i| unsafe { EfiStr16::from_ptr(*prot.argv.add(i)).to_string_lossy() })
            .collect();
        Ok(result)
    }
}

// TODO: Make this into a macro or a derive.
impl RustProtocol for Protocol {
    type RawProtocol = RawProtocol;
    fn get_name() -> &'static str {
        PROTOCOL_NAME
    }
    fn get_guid() -> &'static efi::Guid {
        &PROTOCOL_GUID
    }

    fn init_protocol(mp: Arc<Mutex<Option<ManagedProtocol<RawProtocol>>>>) -> UefiResult<Self> {
        Ok(Self { inner: mp })
    }
}
