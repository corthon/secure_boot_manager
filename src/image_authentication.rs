use alloc::vec::Vec;
use core::fmt;
use r_efi::efi;
use r_efi::efi::Guid;

use crate::util::{DebugBuffer, DebugGuid};
use crate::UefiResult;

pub const EFI_CERT_X509_GUID: Guid = Guid::from_fields(
    0xa5c059a1,
    0x94e4,
    0x4aa7,
    0x87,
    0xb5,
    &[0xab, 0x15, 0x5c, 0x2b, 0xf0, 0x72],
);

pub const EFI_CERT_SHA256_GUID: Guid = Guid::from_fields(
    0xc1c41626,
    0x504c,
    0x4092,
    0xac,
    0xa9,
    &[0x41, 0xf9, 0x36, 0x93, 0x43, 0x28],
);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RawEfiSignatureList {
    signature_type: Guid,
    signature_list_size: u32,
    signature_header_size: u32,
    signature_size: u32,
}
// Have to do this because Guid doesn't support #[repr(packed)]
const RAW_EFI_SIGNATURE_LIST_SIZE: usize =
    core::mem::size_of::<Guid>() + 3 * core::mem::size_of::<u32>();

impl RawEfiSignatureList {
    unsafe fn is_valid(slf: *const Self) -> bool {
        // Check that all internal sizes are less than the full size.
        ((RAW_EFI_SIGNATURE_LIST_SIZE
            + ((*slf).signature_header_size as usize)
            + ((*slf).signature_size as usize))
            <= ((*slf).signature_list_size as usize)) &&
        // Check that the total size is an even multiple of the
        // signature size.
        ((*slf).signature_list_size as usize
            - RAW_EFI_SIGNATURE_LIST_SIZE
            - (*slf).signature_header_size as usize)
            % (*slf).signature_size as usize == 0
    }
}

#[derive(Clone)]
pub struct SignatureListElement {
    pub owner: Guid,
    pub data: Vec<u8>,
}

impl SignatureListElement {
    pub fn from_bytes(buffer: &[u8]) -> UefiResult<Self> {
        if buffer.len() < core::mem::size_of::<Guid>() {
            return Err(efi::Status::BAD_BUFFER_SIZE);
        }

        Ok(SignatureListElement {
            owner: unsafe { *(buffer.as_ptr() as *const Guid) },
            data: Vec::<u8>::from(&buffer[core::mem::size_of::<Guid>()..]),
        })
    }
}

impl fmt::Debug for SignatureListElement {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        if fmtr.alternate() {
            fmtr.write_str("SignatureListElement:\n")?;
            fmtr.write_fmt(format_args!(
                "\tOwner: {:#?}\n",
                DebugGuid::from(self.owner)
            ))?;
            fmtr.write_fmt(format_args!("\tData: 0x{:X?} bytes\n", self.data.len()))?;
            fmtr.write_fmt(format_args!("{:#?}\n", DebugBuffer::new(&self.data)))?;
            Ok(())
        } else {
            fmtr.write_fmt(format_args!(
                "SignatureListElement {{ Owner: {:?} }}",
                DebugGuid::from(self.owner)
            ))
        }
    }
}

#[derive(Clone)]
pub struct SignatureList {
    pub list_type: Guid,
    pub header: Option<Vec<u8>>,
    pub elements: Vec<SignatureListElement>,
}

impl SignatureList {
    pub fn from_bytes(buffer: &[u8]) -> UefiResult<Self> {
        if buffer.len() < RAW_EFI_SIGNATURE_LIST_SIZE {
            return Err(efi::Status::INVALID_PARAMETER);
        }

        let total_size: usize;
        let header_size: usize;
        let signature_size: usize;
        unsafe {
            let raw_ptr = buffer.as_ptr() as *const RawEfiSignatureList;
            // Make sure that the buffer sizes live within a contraint.
            if !RawEfiSignatureList::is_valid(raw_ptr)
                || (*raw_ptr).signature_list_size as usize > buffer.len()
            {
                return Err(efi::Status::BAD_BUFFER_SIZE);
            }

            total_size = (*raw_ptr).signature_list_size as usize;
            header_size = (*raw_ptr).signature_header_size as usize;
            signature_size = (*raw_ptr).signature_size as usize;
        }

        // Create the basic result.
        let mut result = Self {
            list_type: unsafe { *(buffer.as_ptr() as *const Guid) },
            header: match header_size {
                0 => None,
                _ => Some(Vec::<u8>::from(
                    &buffer[RAW_EFI_SIGNATURE_LIST_SIZE..RAW_EFI_SIGNATURE_LIST_SIZE + header_size],
                )),
            },
            elements: Vec::<SignatureListElement>::new(),
        };

        // Add any signature elements.
        let elements_start = RAW_EFI_SIGNATURE_LIST_SIZE + header_size;
        let elements_size = total_size - elements_start;
        for list_index in 0..(elements_size / signature_size) {
            let element_start = elements_start + list_index * signature_size;
            let element_end = element_start + signature_size;
            result.elements.push(SignatureListElement::from_bytes(
                &buffer[element_start..element_end],
            )?);
        }

        Ok(result)
    }
}

impl fmt::Debug for SignatureList {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        if fmtr.alternate() {
            fmtr.write_str("SignatureList:\n")?;
            fmtr.write_fmt(format_args!(
                "\tType: {:#?}\n",
                DebugGuid::from(self.list_type)
            ))?;
            for i in 0..self.elements.len() {
                fmtr.write_fmt(format_args!("{:#?}", self.elements[i]))?;
            }
            Ok(())
        } else {
            fmtr.write_fmt(format_args!(
                "SignatureList {{ Type: {:?}, Count: {:?} }}",
                DebugGuid::from(self.list_type),
                self.elements.len()
            ))
        }
    }
}

#[derive(Clone)]
pub struct SignatureDatabase {
    pub entries: Vec<SignatureList>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::sig_lists;

    #[test]
    fn efi_sig_list_from_bytes_should_fail_for_buffer_too_small() {
        let test_buffer: &[u8] = &[0x62, 0x12, 0x13, 0x14];
        assert!(SignatureList::from_bytes(test_buffer).is_err());
    }

    #[test]
    fn efi_sig_list_should_be_able_to_pread_test_data() {
        let test_list = SignatureList::from_bytes(sig_lists::PK).unwrap();
        assert_eq!(test_list.list_type, EFI_CERT_X509_GUID);
        assert_eq!(test_list.elements.len(), 1);
        println!("{:#?}", test_list);

        let test_list = SignatureList::from_bytes(sig_lists::DBX).unwrap();
        assert_eq!(test_list.list_type, EFI_CERT_SHA256_GUID);
        assert_eq!(test_list.elements.len(), 77);
    }
}
