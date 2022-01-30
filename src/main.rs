#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate allocation;
extern crate uefi;

use alloc::string::String;
use core::cell::RefCell;
use core::ptr::NonNull;
use mu_rust_ex::protocol_wrapper::ProtocolWrapper;
use r_efi::efi;

use core_con_out::println;
use mu_rust_ex::{shell_parameters_protocol, shell_protocol, UefiResult};

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
        let args = shell_params.get_args()?;
        println!("{:?}", args);

        let mut iter = args.iter();
        let mut output_file: Option<String> = None;
        loop {
            let arg = iter.next();
            if arg.is_none() {
                break;
            }

            let arg = arg.unwrap();
            if arg.eq("-o") {
                output_file = Some(String::from(iter.next().unwrap()));
            }
        }

        println!("Output File: {:?}", output_file);

        match output_file {
            None => (),
            Some(of) => {
                let shell = ProtocolWrapper::<shell_protocol::Protocol>::first()?;
                let file_handle = shell.create_file(
                    &of,
                    r_efi::protocols::file::MODE_CREATE | r_efi::protocols::file::MODE_WRITE,
                )?;

                let data: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
                match shell.write_file(file_handle, &data) {
                    Ok(_) => println!("File successfully written!"),
                    Err(e) => println!("Failed to write file! {:?}", e),
                };

                shell.flush_file(file_handle)?;
                shell.close_file(file_handle)?;
            }
        };

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
