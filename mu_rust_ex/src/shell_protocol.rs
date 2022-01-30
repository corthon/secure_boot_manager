// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::protocol_wrapper::{
    ManagedProtocol, ManagedProtocolError as MPError, ManagedProtocolResult as MPResult,
};
use crate::UefiResult;

use r_efi::{efi, eficall, eficall_abi};
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
        *const efi::Char16,
        u64,
        *mut FileHandle,
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
    //   EFI_SHELL_FIND_FILES                      FindFiles;
    //   EFI_SHELL_FIND_FILES_IN_DIR               FindFilesInDir;
    //   EFI_SHELL_GET_FILE_SIZE                   GetFileSize;
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

pub struct Protocol {
    inner: Option<&'static RawProtocol>,
    handle: efi::Handle,
}

impl Protocol {
    pub fn open_file_by_name(&self, name: &str, mode: u64) -> MPResult<FileHandle> {
        let prot = self.inner.ok_or(MPError::Unregistered)?;
        let efi_name = OsString::from(name);
        let mut handle: FileHandle = core::ptr::null_mut();

        let status = (prot.open_file_by_name)(efi_name.as_ptr(), &mut handle as *mut _, mode);

        if !status.is_error() {
            Ok(handle)
        } else {
            Err(MPError::Efi(status))
        }
    }

    pub fn read_file(&self, handle: FileHandle, buffer: &mut [u8]) -> MPResult<usize> {
        let prot = self.inner.ok_or(MPError::Unregistered)?;
        let mut read_size: usize = buffer.len();

        let status = (prot.read_file)(
            handle,
            &mut read_size as *mut _,
            buffer.as_mut_ptr() as *mut _,
        );

        if !status.is_error() {
            Ok(read_size)
        } else {
            Err(MPError::Efi(status))
        }
    }

    pub fn write_file(&self, handle: FileHandle, buffer: &[u8]) -> MPResult<usize> {
        let prot = self.inner.ok_or(MPError::Unregistered)?;
        let mut write_size: usize = buffer.len();

        let status = (prot.write_file)(
            handle,
            &mut write_size as *mut _,
            buffer.as_ptr() as *const _,
        );

        if !status.is_error() {
            Ok(write_size)
        } else {
            Err(MPError::Efi(status))
        }
    }
}

// TODO: Make this into a macro or a derive.
impl ManagedProtocol for Protocol {
    type ProtocolType = Self;

    fn get_name() -> &'static str {
        PROTOCOL_NAME
    }
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
