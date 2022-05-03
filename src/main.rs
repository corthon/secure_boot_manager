#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate panic;
extern crate uefi_bs_allocator as uefi_allocator;

use alloc::string::String;
use core::ptr::NonNull;
use r_efi::efi;

use core_con_out::println;
use mu_rust_ex::{
    con_in::ConIn, protocol_utility::RustProtocol,
    shell_parameters_protocol::Protocol as ShellParametersProtocol,
    shell_protocol::Protocol as ShellProtocol, UefiResult,
};

#[allow(dead_code)]
struct AppInstance {
    h: efi::Handle,
    st: NonNull<efi::SystemTable>,
}

// TODO: Possibly make this AppInstance a shared static.
//       There's tradeoffs to this, but the handle might be needed in several places.

#[allow(dead_code)]
impl AppInstance {
    pub fn init(h: efi::Handle, st: *mut efi::SystemTable) -> UefiResult<Self> {
        let st = NonNull::new(st).ok_or(efi::Status::INVALID_PARAMETER)?;
        Ok(Self { st, h })
    }

    pub fn main(&mut self) -> UefiResult<()> {
        println!("WELCOME TO THE APP!");

        let shell = ShellProtocol::first()?;
        let shell_params = ShellParametersProtocol::by_handle(self.h)?;
        let args = shell_params.get_args()?;
        println!("{:?}", args);

        let mut iter = args.iter();
        let mut output_file: Option<String> = None;
        let mut input_file: Option<String> = None;
        loop {
            match iter.next() {
                None => break,
                Some(arg) => {
                    if arg.eq("-o") {
                        output_file = iter.next().map(|arg| String::from(arg));
                    } else if arg.eq("-i") {
                        input_file = iter.next().map(|arg| String::from(arg));
                    }
                }
            }
        }

        println!("Output File: {:?}", output_file);
        println!("Input File: {:?}", input_file);

        if let Some(ref ofile) = output_file {
            let mut file = shell.create_file(
                ofile,
                r_efi::protocols::file::MODE_CREATE | r_efi::protocols::file::MODE_WRITE,
            )?;

            let data: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
            match file.write(&data) {
                Ok(_) => println!("File successfully written!"),
                Err(e) => println!("Failed to write file! {:?}", e),
            };
        }

        if let Some(ref ifile) = input_file {
            let file = shell.open_file_by_name(ifile, r_efi::protocols::file::MODE_READ)?;
            println!("File size: {}", file.get_size()?);
            let bytes = file.read_count(file.get_size()?)?;
            println!("BYTES! {:?}", bytes);
        }

        let con_in = unsafe { ConIn::new(self.st.as_ptr())? };
        for _ in 0..5 {
            println!("Output char '{}'", con_in.get_char()?);
        }

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
        let _ = uefi_allocator::init(st);
        // Setup the Ex lib.
        let _ = mu_rust_ex::init_lib(st);
    }
    let mut app = AppInstance::init(h, st).unwrap();

    match app.main() {
        Ok(_) => efi::Status::SUCCESS,
        Err(err) => err,
    }
}
