#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate allocation;
extern crate uefi;

use core::cell::RefCell;
use core::ptr::NonNull;
use mu_rust_ex::protocol_wrapper::ProtocolWrapper;
use r_efi::efi;

use core_con_out::println;
use mu_rust_ex::variable::EfiVariable;
use mu_rust_ex::{auth_variable, shell_parameters_protocol, variable, UefiResult};

#[allow(dead_code)]
struct AppInstance {
    h: efi::Handle,
    st: RefCell<NonNull<efi::SystemTable>>,
}

// TODO: Possibly make this AppInstance a shared static.
//       There's tradeoffs to this, but the handle might be needed in several places.

#[allow(dead_code)]
impl AppInstance {
    pub fn init(h: efi::Handle, st: *mut efi::SystemTable) -> UefiResult<Self> {
        let st_inner = NonNull::new(st).ok_or(efi::Status::INVALID_PARAMETER)?;
        Ok(Self {
            st: RefCell::new(st_inner),
            h,
        })
    }

    pub fn main(&mut self) -> UefiResult<()> {
        println!("WELCOME TO THE APP!");

        let shell_params =
            ProtocolWrapper::<shell_parameters_protocol::Protocol>::by_handle(self.h)?;
        let args = shell_params.get_args();
        println!("{:?}", args);

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
