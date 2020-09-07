#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
extern crate panic;
extern crate uefi;
#[cfg(not(test))]
extern crate allocation;

use our_efi::efi;
use string::OsString;

struct AppInstance {
  h: efi::Handle,
  st: core::ptr::NonNull<efi::SystemTable>,
}

impl AppInstance {
  pub fn init(h: efi::Handle, st: *mut efi::SystemTable) -> Result<Self, efi::Status> {
    if !st.is_null() {
      Ok(Self { h, st: core::ptr::NonNull::new(st).unwrap() })
    } else {
      Err(efi::Status::INVALID_PARAMETER)
    }
  }

  fn print(&mut self, out_string: &str) {
    unsafe {
      let con_out = (*self.st.as_ptr()).con_out;
      ((*con_out).output_string)(con_out, OsString::from(out_string).as_mut_ptr() as *mut efi::Char16);
    }
  }

  pub fn main(&mut self) -> efi::Status {
    let source_string = "This is my string.\r\nThere are many strings like it, but this one is mine.\r\n";
    self.print(source_string);

    efi::Status::SUCCESS
  }
}

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

#[cfg(test)]
mod tests {
  use string::OsString;

  #[test]
  fn os_string_should_have_values() {
      let str_str = "This is my test string.";
      println!("{}", str_str);
  }
}