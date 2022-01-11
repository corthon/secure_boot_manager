use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::fmt;
use core::fmt::Write;
use r_efi::efi;

use crate::image_authentication::{EFI_CERT_SHA256_GUID, EFI_CERT_X509_GUID};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct DebugGuid(efi::Guid);
impl fmt::Debug for DebugGuid {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        let inner_string = match self.0 {
            EFI_CERT_X509_GUID => String::from("EFI_CERT_X509_GUID"),
            EFI_CERT_SHA256_GUID => String::from("EFI_CERT_SHA256_GUID"),
            _ => {
                let guid_bytes = self.0.as_bytes();
                let mut buffer = String::new();
                buffer.write_fmt(format_args!(
                    "UNKNOWN({:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X})",
                    u32::from_le_bytes(guid_bytes[..4].try_into().unwrap()),
                    u16::from_le_bytes(guid_bytes[4..6].try_into().unwrap()),
                    u16::from_le_bytes(guid_bytes[6..8].try_into().unwrap()),
                    guid_bytes[8],
                    guid_bytes[9],
                    guid_bytes[10],
                    guid_bytes[11],
                    guid_bytes[12],
                    guid_bytes[13],
                    guid_bytes[14],
                    guid_bytes[15],
                ))?;
                buffer
            }
        };
        fmtr.write_fmt(format_args!("Guid {{ {} }}", inner_string))
    }
}
impl From<efi::Guid> for DebugGuid {
    fn from(from: efi::Guid) -> Self {
        Self(from)
    }
}

#[derive(Clone)]
pub struct DebugBuffer<'a>(&'a Vec<u8>);
impl<'a> DebugBuffer<'a> {
    pub fn new(data: &'a Vec<u8>) -> Self {
        Self(data)
    }
}
impl<'a> fmt::Debug for DebugBuffer<'a> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        let last_row_count = self.0.len() % 0x10;
        let last_row = self.0.len() / 0x10
            + match last_row_count {
                0 => 0,
                _ => 1,
            };
        for row in 0..last_row {
            fmtr.write_fmt(format_args!("0x{:04X}:  ", row * 0x10))?;
            for index in 0..0x10 {
                if row != (last_row - 1) || last_row_count == 0 || index < last_row_count {
                    fmtr.write_fmt(format_args!("{:02X} ", self.0[row * 0x10 + index]))?;
                } else {
                    fmtr.write_str("   ")?;
                }
                if index == 7 {
                    fmtr.write_str("- ")?;
                }
            }
            fmtr.write_str("  ")?;
            for index in 0..0x10 {
                if row != (last_row - 1) || last_row_count == 0 || index < last_row_count {
                    let chr = self.0[row * 0x10 + index];
                    fmtr.write_fmt(format_args!(
                        "{}",
                        match chr {
                            0x20..=0x7F => char::from(chr),
                            _ => '.',
                        }
                    ))?;
                } else {
                    fmtr.write_str(" ")?;
                }
            }
            fmtr.write_str("\n")?;
        }
        Ok(())
    }
}
