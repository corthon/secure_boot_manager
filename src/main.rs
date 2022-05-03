#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate panic;
extern crate uefi_bs_allocator as uefi_allocator;

use alloc::string::String;
use core::fmt::Write;
use core::ptr::NonNull;
use menu::*;
use r_efi::efi;

use core_con_out::{print, println};
use mu_rust_ex::{
    con_in::ConIn, con_in::InputKey, protocol_utility::RustProtocol,
    shell_parameters_protocol::Protocol as ShellParametersProtocol,
    shell_protocol::Protocol as ShellProtocol, UefiResult,
};

struct PrintOutput;
impl core::fmt::Write for PrintOutput {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

const ROOT_MENU: Menu<PrintOutput> = Menu {
    label: "root",
    items: &[
        &Item {
            item_type: ItemType::Callback {
                function: select_bar,
                parameters: &[],
            },
            command: "bar",
            help: Some("fandoggles a bar"),
        },
        &Item {
            item_type: ItemType::Menu(&Menu {
                label: "sub",
                items: &[
                    &Item {
                        item_type: ItemType::Callback {
                            function: select_baz,
                            parameters: &[],
                        },
                        command: "baz",
                        help: Some("thingamobob a baz"),
                    },
                    &Item {
                        item_type: ItemType::Callback {
                            function: select_quux,
                            parameters: &[],
                        },
                        command: "quux",
                        help: Some("maximum quux"),
                    },
                ],
                entry: Some(enter_sub),
                exit: Some(exit_sub),
            }),
            command: "sub",
            help: Some("enter sub-menu"),
        },
    ],
    entry: Some(enter_root),
    exit: Some(exit_root),
};

fn enter_root(_menu: &Menu<PrintOutput>, context: &mut PrintOutput) {
    writeln!(context, "In enter_root").unwrap();
}

fn exit_root(_menu: &Menu<PrintOutput>, context: &mut PrintOutput) {
    writeln!(context, "In exit_root").unwrap();
}

fn select_bar<'a>(
    _menu: &Menu<PrintOutput>,
    _item: &Item<PrintOutput>,
    args: &[&str],
    context: &mut PrintOutput,
) {
    writeln!(context, "In select_bar. Args = {:?}", args).unwrap();
}

fn enter_sub(_menu: &Menu<PrintOutput>, context: &mut PrintOutput) {
    writeln!(context, "In enter_sub").unwrap();
}

fn exit_sub(_menu: &Menu<PrintOutput>, context: &mut PrintOutput) {
    writeln!(context, "In exit_sub").unwrap();
}

fn select_baz<'a>(
    _menu: &Menu<PrintOutput>,
    _item: &Item<PrintOutput>,
    args: &[&str],
    context: &mut PrintOutput,
) {
    writeln!(context, "In select_baz: Args = {:?}", args).unwrap();
}

fn select_quux<'a>(
    _menu: &Menu<PrintOutput>,
    _item: &Item<PrintOutput>,
    args: &[&str],
    context: &mut PrintOutput,
) {
    writeln!(context, "In select_quux: Args = {:?}", args).unwrap();
}

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
        let mut buffer = [0u8; 64];
        let mut r = Runner::new(&ROOT_MENU, &mut buffer, PrintOutput);
        loop {
            match con_in.get_char()? {
                InputKey::Char('\n') => r.input_byte(b'\n'),
                InputKey::Char(c) => {
                    let mut buf = [0; 4];
                    for b in c.encode_utf8(&mut buf).bytes() {
                        r.input_byte(b);
                    }
                }
                InputKey::Ctrl(0x17) => break,
                _ => (),
            }
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
