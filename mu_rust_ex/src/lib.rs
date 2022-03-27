#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[macro_use]
extern crate lazy_static;

// TODO: Can any of this be gated behind an "alloc" feature flag?

pub mod auth_variable;
pub mod boot;
pub mod image_authentication;
pub mod protocol_utility;
pub mod runtime;
pub mod shell_parameters_protocol;
pub mod shell_protocol;
#[cfg(test)]
mod test_data;
pub mod util;
pub mod variable;

use r_efi::efi;

#[allow(unused)]
use core_con_out::println;

pub type UefiResult<T> = Result<T, efi::Status>;

pub unsafe fn init_lib(st: *mut efi::SystemTable) -> UefiResult<()> {
    boot::init_uefi_bs(st)?;
    runtime::init_uefi_rs(st)?;
    Ok(())
}
