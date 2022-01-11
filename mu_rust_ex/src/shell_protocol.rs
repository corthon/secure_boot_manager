// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use r_efi::efi::Guid;
use r_efi::{eficall, eficall_abi};

pub const PROTOCOL_GUID: Guid = Guid::from_fields(
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
        *const r_efi::base::Handle,
        *const r_efi::efi::Char16,
        *const *const r_efi::efi::Char16,
        *mut r_efi::base::Status,
    ) -> r_efi::base::Status},
    pub get_env: eficall! {fn(
        *const r_efi::efi::Char16,
    ) -> *const r_efi::efi::Char16},
    pub set_env: eficall! {fn(
        *const r_efi::efi::Char16,
        *const r_efi::efi::Char16,
        r_efi::efi::Boolean,
    ) -> r_efi::base::Status},
    pub get_alias: eficall! {fn(
        *const r_efi::efi::Char16,
        *mut r_efi::efi::Boolean,
    ) -> *const r_efi::efi::Char16},
    pub set_alias: eficall! {fn(
        *const r_efi::efi::Char16,
        *const r_efi::efi::Char16,
        r_efi::efi::Boolean,
        r_efi::efi::Boolean,
    ) -> r_efi::base::Status},
    pub get_help_text: eficall! {fn(
        *const r_efi::efi::Char16,
        *const r_efi::efi::Char16,
        *mut *mut r_efi::efi::Char16,
    ) -> r_efi::base::Status},
    pub get_device_path_from_map: eficall! {fn(
        *const r_efi::efi::Char16,
    ) -> *const r_efi::efi::protocols::device_path::Protocol},
    pub get_map_from_device_path: eficall! {fn(
        *mut r_efi::efi::protocols::device_path::Protocol,
    ) -> *const r_efi::efi::Char16},
    pub get_device_path_from_file_path: eficall! {fn(
        *const r_efi::efi::Char16,
    ) -> *const r_efi::efi::protocols::device_path::Protocol},
    pub get_file_path_from_device_path: eficall! {fn(
        *const r_efi::efi::protocols::device_path::Protocol,
    ) -> *const r_efi::efi::Char16},
    pub set_map: eficall! {fn(
        *const r_efi::efi::protocols::device_path::Protocol,
        *const r_efi::efi::Char16,
    ) -> r_efi::base::Status},
    pub get_cur_dir: eficall! {fn(
        *const r_efi::efi::Char16,
    ) -> *const r_efi::efi::Char16},
    pub set_cur_dir: eficall! {fn(
        *const r_efi::efi::Char16,
        *const r_efi::efi::Char16,
    ) -> r_efi::base::Status},
    pub open_file_list: eficall! {fn(
        *const r_efi::efi::Char16,
        u64,
        *mut *mut FileInfo,
    ) -> r_efi::base::Status},
    pub free_file_list: eficall! {fn(
        *mut *mut FileInfo,
    ) -> r_efi::base::Status},
    pub remove_dup_in_file_list: eficall! {fn(
        *mut *mut FileInfo,
    ) -> r_efi::base::Status},
    pub batch_is_active: eficall! {fn() -> r_efi::efi::Boolean},
    pub is_root_shell: eficall! {fn() -> r_efi::efi::Boolean},
    pub enable_page_break: eficall! {fn() -> ()},
    pub disable_page_break: eficall! {fn() -> ()},
    pub get_page_break: eficall! {fn() -> r_efi::efi::Boolean},
    pub get_device_name: eficall! {fn(
        r_efi::base::Handle,
        DeviceNameFlags,
        *const r_efi::efi::Char8,
        *mut *mut r_efi::efi::Char16,
    ) -> r_efi::base::Status},
    pub get_file_info: eficall! {fn(
        FileHandle,
    ) -> *const r_efi::efi::protocols::file::Info},
    //   EFI_SHELL_SET_FILE_INFO                   SetFileInfo;
    //   EFI_SHELL_OPEN_FILE_BY_NAME               OpenFileByName;
    //   EFI_SHELL_CLOSE_FILE                      CloseFile;
    //   EFI_SHELL_CREATE_FILE                     CreateFile;
    //   EFI_SHELL_READ_FILE                       ReadFile;
    //   EFI_SHELL_WRITE_FILE                      WriteFile;
    //   EFI_SHELL_DELETE_FILE                     DeleteFile;
    //   EFI_SHELL_DELETE_FILE_BY_NAME             DeleteFileByName;
    //   EFI_SHELL_GET_FILE_POSITION               GetFilePosition;
    //   EFI_SHELL_SET_FILE_POSITION               SetFilePosition;
    //   EFI_SHELL_FLUSH_FILE                      FlushFile;
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
    status: r_efi::base::Status,
    full_name: *const r_efi::efi::Char16,
    file_name: *const r_efi::efi::Char16,
    handle: FileHandle,
    info: *const r_efi::efi::protocols::file::Info,
}

pub type FileHandle = *mut core::ffi::c_void;

pub type DeviceNameFlags = u32;
pub const DEVICE_NAME_USE_COMPONENT_NAME: DeviceNameFlags = 0x00000001;
pub const DEVICE_NAME_USE_DEVICE_PATH: DeviceNameFlags = 0x00000002;

// Currently being designed with extra code that might get wrapped in a trait?
// Something about being able to Open and Close automatically?
// pub struct Protocol {
//     handle: r_efi::efi::Handle,
//     agent_handle: r_efi::efi::Handle,
//     guid: r_efi::efi::Guid, // Is this necessary? I think get_guid() could be a trait.
//     // Theoretically, if we could get the notificaitons working, the Option
//     // could be set to None on unexpected disconnect.
//     inner: Option<RawProtocol>,
// }
