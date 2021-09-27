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

pub type AppResult<T> = Result<T, r_efi::efi::Status>;

#[allow(dead_code)]
struct AppInstance {
    h: efi::Handle,
    st: core::ptr::NonNull<efi::SystemTable>,
}

#[allow(dead_code)]
impl AppInstance {
    pub fn init(h: efi::Handle, st: *mut efi::SystemTable) -> Result<Self, efi::Status> {
        if !st.is_null() {
            Ok(Self {
                h,
                st: core::ptr::NonNull::new(st).unwrap(),
            })
        } else {
            Err(efi::Status::INVALID_PARAMETER)
        }
    }

    fn print(&mut self, out_string: &str) {
        unsafe {
            let con_out = (*self.st.as_ptr()).con_out;
            ((*con_out).output_string)(
                con_out,
                OsString::from(out_string).as_mut_ptr() as *mut efi::Char16,
            );
        }
    }

    pub fn main(&mut self) -> efi::Status {
        let source_string =
            "This is my string.\r\nThere are many strings like it, but this one is mine.\r\n";
        self.print(source_string);

        let rs = runtime::RuntimeServices::new(self.st.as_ptr());
        let mut result = rs.get_variable("Boot0000", &runtime::EFI_GLOBAL_VARIABLE_GUID);
        self.print(&alloc::format!("{:?}\r\n", result));
        result = rs.get_variable("SetupMode", &runtime::EFI_GLOBAL_VARIABLE_GUID);
        self.print(&alloc::format!("{:?}\r\n", result));
        result = rs.get_variable("SignatureSupport", &runtime::EFI_GLOBAL_VARIABLE_GUID);
        self.print(&alloc::format!("{:?}\r\n", result));
        result = rs.get_variable("OsIndicationsSupported", &runtime::EFI_GLOBAL_VARIABLE_GUID);
        self.print(&alloc::format!("{:?}\r\n", result));
        result = rs.get_variable("NotAVar", &runtime::EFI_GLOBAL_VARIABLE_GUID);
        self.print(&alloc::format!("{:?}\r\n", result));

        efi::Status::SUCCESS
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
    app.main()
}
