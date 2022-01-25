// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::protocol_wrapper::{
    ManagedProtocol, ManagedProtocolError as MPError, ManagedProtocolResult as MPResult,
};
use crate::shell_protocol;
use r_efi::efi;
use r_efi_string::str16::EfiStr16;

use crate::{println, UefiResult};

pub const PROTOCOL_GUID: efi::Guid = efi::Guid::from_fields(
    0x752f3136,
    0x4e16,
    0x4fdc,
    0xa2,
    0x2a,
    &[0xe5, 0xf4, 0x68, 0x12, 0xf4, 0xca],
);

#[repr(C)]
struct RawProtocol {
    pub argv: *const *const efi::Char16,
    pub argc: usize,
    pub std_in: shell_protocol::FileHandle,
    pub std_out: shell_protocol::FileHandle,
    pub std_err: shell_protocol::FileHandle,
}

pub struct Protocol {
    inner: Option<&'static RawProtocol>,
    handle: efi::Handle,
}

impl Protocol {
    // NOTE: It's important that these all return ManagedProtocolError::Unregistered
    //       if the Option has been taken.
    pub fn get_args(&self) -> MPResult<Vec<String>> {
        let prot = self.inner.ok_or(MPError::Unregistered)?;

        let result: Vec<String> = (0..prot.argc)
            .map(|i| unsafe { EfiStr16::from_ptr(*prot.argv.add(i)).to_string_lossy() })
            .collect();
        Ok(result)
    }
}

// TODO: Make this into a macro or a derive.
impl ManagedProtocol for Protocol {
    type ProtocolType = Self;

    fn get_guid() -> &'static efi::Guid {
        &PROTOCOL_GUID
    }

    fn init_protocol(
        prot: *mut core::ffi::c_void,
        handle: efi::Handle,
    ) -> UefiResult<Self::ProtocolType> {
        Ok(Self {
            inner: unsafe { (prot as *const RawProtocol).as_ref() },
            handle,
        })
    }

    fn get_handle(&self) -> efi::Handle {
        self.handle
    }

    fn deinit_protocol(&mut self) {
        self.inner = None;
    }
}
