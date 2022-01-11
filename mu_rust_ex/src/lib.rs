#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod auth_variable;
pub mod boot;
pub mod image_authentication;
pub mod runtime;
pub mod shell_protocol;
pub mod util;

pub type UefiResult<T> = Result<T, r_efi::efi::Status>;
