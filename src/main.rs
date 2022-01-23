#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate allocation;
extern crate uefi;

use core::cell::RefCell;
use core::ptr::NonNull;
use r_efi::efi;

use core_con_out::println;
use mu_rust_ex::variable::EfiVariable;
use mu_rust_ex::{auth_variable, variable, UefiResult};

#[allow(dead_code)]
struct AppInstance {
    h: efi::Handle,
    st: RefCell<NonNull<efi::SystemTable>>,
}

// TODO: Possibly make this AppInstance a shared static.
//       There's tradeoffs to this, but the handle might be needed in several places.

#[allow(dead_code)]
impl AppInstance {
    pub fn init(h: efi::Handle, st: *mut efi::SystemTable) -> Result<Self, efi::Status> {
        if !st.is_null() {
            Ok(Self {
                h,
                st: RefCell::new(NonNull::new(st).unwrap()),
            })
        } else {
            Err(efi::Status::INVALID_PARAMETER)
        }
    }

    pub fn main(&mut self) -> UefiResult<()> {
        println!("WELCOME TO THE APP!");

        let ret = EfiVariable::get_variable(
            auth_variable::EFI_IMAGE_SECURITY_DATABASE,
            &auth_variable::EFI_IMAGE_SECURITY_DATABASE_GUID,
        );
        println!("{:?}", ret);
        let ret = EfiVariable::get_variable(
            auth_variable::EFI_IMAGE_SECURITY_DATABASE1,
            &auth_variable::EFI_IMAGE_SECURITY_DATABASE_GUID,
        );
        println!("{:?}", ret);
        let ret = EfiVariable::get_variable("PK", &variable::EFI_GLOBAL_VARIABLE_GUID);
        println!("{:?}", ret);
        let ret = EfiVariable::get_variable("KEK", &variable::EFI_GLOBAL_VARIABLE_GUID);
        println!("{:?}", ret);

        Ok(())
    }
}

#[cfg(not(test))]
#[export_name = "efi_main"]
pub extern "C" fn app_entry(h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    use core_con_out::ConOut;

    unsafe {
        // Set up the console.
        let _ = ConOut::init(st);
        // Set up the allocator.
        allocation::init(st);
        // Set up BootServices.
        uefi::services::boot::init_by_st(st);
        // Setup the Ex lib.
        let _ = mu_rust_ex::init_lib(st);
    }
    let mut app = AppInstance::init(h, st).unwrap();

    match app.main() {
        Ok(_) => efi::Status::SUCCESS,
        Err(err) => err,
    }
}
