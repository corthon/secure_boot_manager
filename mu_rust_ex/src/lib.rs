#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod auth_variable;
pub mod boot;
pub mod image_authentication;
pub mod protocol_wrapper;
pub mod runtime;
pub mod shell_protocol;
pub mod util;
pub mod variable;

use r_efi::efi;

pub type UefiResult<T> = Result<T, efi::Status>;

pub unsafe fn init_lib(st: *mut efi::SystemTable) -> UefiResult<()> {
    boot::init_uefi_bs(st)?;
    runtime::init_uefi_rs(st)?;
    Ok(())
}
