#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
#[cfg(not(test))]
extern crate allocation;
#[cfg(not(test))]
extern crate panic;
extern crate uefi;

mod image_authentication;
mod boot;
mod runtime;
mod test_data;
mod util;
mod shell_protocol;

use r_efi::efi;
use string::OsString;
use core::ptr::NonNull;
use core::cell::RefCell;

pub type UefiResult<T> = Result<T, r_efi::efi::Status>;

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

    pub fn print(&self, out_string: &str) {
        let st = self.st.borrow_mut();
        unsafe {
            let con_out = (*st.as_ptr()).con_out;
            ((*con_out).output_string)(
                con_out,
                OsString::from(out_string).as_mut_ptr() as *mut efi::Char16,
            );
        }
    }

    pub fn main(&mut self) -> UefiResult<()> {
        self.print("WELCOME TO THE APP!");

        let rs = runtime::RuntimeServices::new((*self.st.borrow()).as_ptr());
        boot::BootServices::init((*self.st.borrow()).as_ptr())?;

        Ok(())
    }
}

#[cfg(not(test))]
#[export_name = "efi_main"]
pub extern "C" fn app_entry(h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    unsafe {
        // Set up the allocator.
        allocation::init(st);
        // Set up BootServices.
        uefi::services::boot::init_by_st(st);
    }
    let mut app = AppInstance::init(h, st).unwrap();

    match app.main() {
        Ok(_) => efi::Status::SUCCESS,
        Err(err) => err
    }
}
