// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::protocol_utility::{
    ManagedProtocol, RustProtocol, RustProtocolError as RPError, RustProtocolResult as RPResult,
};
use crate::UefiResult;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;

use r_efi::{efi, eficall, eficall_abi};
use spin::Mutex;
use string::OsString;

pub const PROTOCOL_NAME: &str = "EfiShellProtocol";
pub const PROTOCOL_GUID: efi::Guid = efi::Guid::from_fields(
    0x6302d008,
    0x7f9b,
    0x4f30,
    0x87,
    0xac,
    &[0x60, 0xc9, 0xfe, 0xf5, 0xda, 0x4e],
);

#[repr(C)]
pub struct RawProtocol {
    pub execute: eficall! {fn(
        *const efi::Handle,
        *const efi::Char16,
        *const *const efi::Char16,
        *mut efi::Status,
    ) -> efi::Status},
    pub get_env: eficall! {fn(
        *const efi::Char16,
    ) -> *const efi::Char16},
    pub set_env: eficall! {fn(
        *const efi::Char16,
        *const efi::Char16,
        efi::Boolean,
    ) -> efi::Status},
    pub get_alias: eficall! {fn(
        *const efi::Char16,
        *mut efi::Boolean,
    ) -> *const efi::Char16},
    pub set_alias: eficall! {fn(
        *const efi::Char16,
        *const efi::Char16,
        efi::Boolean,
        efi::Boolean,
    ) -> efi::Status},
    pub get_help_text: eficall! {fn(
        *const efi::Char16,
        *const efi::Char16,
        *mut *mut efi::Char16,
    ) -> efi::Status},
    pub get_device_path_from_map: eficall! {fn(
        *const efi::Char16,
    ) -> *const efi::protocols::device_path::Protocol},
    pub get_map_from_device_path: eficall! {fn(
        *mut efi::protocols::device_path::Protocol,
    ) -> *const efi::Char16},
    pub get_device_path_from_file_path: eficall! {fn(
        *const efi::Char16,
    ) -> *const efi::protocols::device_path::Protocol},
    pub get_file_path_from_device_path: eficall! {fn(
        *const efi::protocols::device_path::Protocol,
    ) -> *const efi::Char16},
    pub set_map: eficall! {fn(
        *const efi::protocols::device_path::Protocol,
        *const efi::Char16,
    ) -> efi::Status},
    pub get_cur_dir: eficall! {fn(
        *const efi::Char16,
    ) -> *const efi::Char16},
    pub set_cur_dir: eficall! {fn(
        *const efi::Char16,
        *const efi::Char16,
    ) -> efi::Status},
    pub open_file_list: eficall! {fn(
        *const efi::Char16,
        u64,
        *mut *mut FileInfo,
    ) -> efi::Status},
    pub free_file_list: eficall! {fn(
        *mut *mut FileInfo,
    ) -> efi::Status},
    pub remove_dup_in_file_list: eficall! {fn(
        *mut *mut FileInfo,
    ) -> efi::Status},
    pub batch_is_active: eficall! {fn() -> efi::Boolean},
    pub is_root_shell: eficall! {fn() -> efi::Boolean},
    pub enable_page_break: eficall! {fn() -> ()},
    pub disable_page_break: eficall! {fn() -> ()},
    pub get_page_break: eficall! {fn() -> efi::Boolean},
    pub get_device_name: eficall! {fn(
        efi::Handle,
        DeviceNameFlags,
        *const efi::Char8,
        *mut *mut efi::Char16,
    ) -> efi::Status},
    pub get_file_info: eficall! {fn(
        FileHandle,
    ) -> *const efi::protocols::file::Info},
    pub set_file_info: eficall! {fn(
        FileHandle,
        *const efi::protocols::file::Info,
    ) -> efi::Status},
    pub open_file_by_name: eficall! {fn(
        *const efi::Char16,
        *mut FileHandle,
        u64,
    ) -> efi::Status},
    pub close_file: eficall! {fn(
        FileHandle,
    ) -> efi::Status},
    pub create_file: eficall! {fn(
        *const efi::Char16,     // FileName
        u64,                    // FileAttribs
        *mut FileHandle,        // FileHandle
    ) -> efi::Status},
    pub read_file: eficall! {fn(
        FileHandle,
        *mut usize,
        *mut core::ffi::c_void,
    ) -> efi::Status},
    pub write_file: eficall! {fn(
        FileHandle,
        *mut usize,
        *const core::ffi::c_void,
    ) -> efi::Status},
    pub delete_file: eficall! {fn(
        FileHandle,
    ) -> efi::Status},
    pub delete_file_by_name: eficall! {fn(
        *const efi::Char16,
    ) -> efi::Status},
    pub get_file_position: eficall! {fn(
        FileHandle,
        *mut u64,
    ) -> efi::Status},
    pub set_file_position: eficall! {fn(
        FileHandle,
        u64,
    ) -> efi::Status},
    pub flush_file: eficall! {fn(
        FileHandle,
    ) -> efi::Status},
    pub find_files: eficall! {fn(
        *const efi::Char16,
        *mut *mut FileInfo,
    ) -> efi::Status},
    pub find_files_in_dir: eficall! {fn(
        FileHandle,
        *mut *mut FileInfo,
    ) -> efi::Status},
    pub get_file_size: eficall! {fn(
        FileHandle,
        *mut u64,
    ) -> efi::Status},
    //   EFI_SHELL_OPEN_ROOT                       OpenRoot;
    //   EFI_SHELL_OPEN_ROOT_BY_HANDLE             OpenRootByHandle;
    //   EFI_EVENT                                 ExecutionBreak;
    //   UINT32                                    MajorVersion;
    //   UINT32                                    MinorVersion;
    //   // Added for Shell 2.1
    //   EFI_SHELL_REGISTER_GUID_NAME              RegisterGuidName;
    //   EFI_SHELL_GET_GUID_NAME                   GetGuidName;
    //   EFI_SHELL_GET_GUID_FROM_NAME              GetGuidFromName;
    //   EFI_SHELL_GET_ENV_EX                      GetEnvEx;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ListEntry {
    flink: *const ListEntry,
    blink: *const ListEntry,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileInfo {
    link: ListEntry,
    status: efi::Status,
    full_name: *const efi::Char16,
    file_name: *const efi::Char16,
    handle: FileHandle,
    info: *const efi::protocols::file::Info,
}

pub type FileHandle = *mut core::ffi::c_void;

pub type DeviceNameFlags = u32;
pub const DEVICE_NAME_USE_COMPONENT_NAME: DeviceNameFlags = 0x00000001;
pub const DEVICE_NAME_USE_DEVICE_PATH: DeviceNameFlags = 0x00000002;

#[derive(Clone)]
pub struct Protocol {
    inner: Arc<Mutex<Option<ManagedProtocol<RawProtocol>>>>,
}

impl Protocol {
    pub fn create_file(&self, name: &str, mode: u64) -> RPResult<ShellFile> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;
        let efi_name = OsString::from(name);
        let mut handle: FileHandle = core::ptr::null_mut();

        let status = (prot.create_file)(efi_name.as_ptr(), mode, &mut handle as *mut _);

        if !status.is_error() {
            Ok(ShellFile {
                handle,
                protocol: self.clone(),
            })
        } else {
            Err(RPError::Efi(status))
        }
    }

    pub fn open_file_by_name(&self, name: &str, mode: u64) -> RPResult<ShellFile> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;
        let efi_name = OsString::from(name);
        let mut handle: FileHandle = core::ptr::null_mut();

        let status = (prot.open_file_by_name)(efi_name.as_ptr(), &mut handle as *mut _, mode);

        if !status.is_error() {
            Ok(ShellFile {
                handle,
                protocol: self.clone(),
            })
        } else {
            Err(RPError::Efi(status))
        }
    }

    fn read_file(&self, handle: FileHandle, buffer: &mut [u8]) -> RPResult<usize> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;
        let mut read_size: usize = buffer.len();

        let status = (prot.read_file)(
            handle,
            &mut read_size as *mut _,
            buffer.as_mut_ptr() as *mut _,
        );

        if !status.is_error() {
            Ok(read_size)
        } else {
            Err(RPError::Efi(status))
        }
    }

    fn write_file(&self, handle: FileHandle, buffer: &[u8]) -> RPResult<usize> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;
        let mut write_size: usize = buffer.len();

        let status = (prot.write_file)(
            handle,
            &mut write_size as *mut _,
            buffer.as_ptr() as *const _,
        );

        if !status.is_error() {
            Ok(write_size)
        } else {
            Err(RPError::Efi(status))
        }
    }

    fn flush_file(&self, handle: FileHandle) -> RPResult<()> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;

        let status = (prot.flush_file)(handle);

        if !status.is_error() {
            Ok(())
        } else {
            Err(RPError::Efi(status))
        }
    }

    fn close_file(&self, handle: FileHandle) -> RPResult<()> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;

        let status = (prot.close_file)(handle);

        if !status.is_error() {
            Ok(())
        } else {
            Err(RPError::Efi(status))
        }
    }

    fn get_file_size(&self, handle: FileHandle) -> RPResult<usize> {
        let prot_guard = self.inner.lock();
        let prot = prot_guard.as_ref().ok_or(RPError::Unregistered)?;

        let mut out_size: u64 = 0;
        let status = (prot.get_file_size)(handle, &mut out_size as *mut _);

        if !status.is_error() {
            Ok(out_size
                .try_into()
                .map_err(|_| RPError::Efi(efi::Status::LOAD_ERROR))?)
        } else {
            Err(RPError::Efi(status))
        }
    }
}

pub struct ShellFile {
    protocol: Protocol,
    handle: FileHandle,
}
impl ShellFile {
    pub fn read(&self, buffer: &mut [u8]) -> RPResult<usize> {
        self.protocol.read_file(self.handle, buffer)
    }
    pub fn read_count(&self, count: usize) -> RPResult<Vec<u8>> {
        let actual_count = core::cmp::min(count, self.get_size()?);
        let mut buffer = Vec::<u8>::with_capacity(actual_count);
        // Init the size pre-emptively to allow the read into the buffer.
        unsafe { buffer.set_len(actual_count) };

        let read_size = self.read(&mut buffer)?;
        unsafe { buffer.set_len(read_size) };

        Ok(buffer)
    }
    pub fn write(&mut self, buffer: &[u8]) -> RPResult<usize> {
        self.protocol.write_file(self.handle, buffer)
    }
    pub fn flush(&mut self) -> RPResult<()> {
        self.protocol.flush_file(self.handle)
    }
    pub fn get_size(&self) -> RPResult<usize> {
        self.protocol.get_file_size(self.handle)
    }
}
impl Drop for ShellFile {
    fn drop(&mut self) {
        _ = self.protocol.flush_file(self.handle);
        _ = self.protocol.close_file(self.handle);
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
