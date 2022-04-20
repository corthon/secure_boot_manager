use alloc::vec::Vec;

use r_efi::efi;
use string::OsString;

use crate::{variable::EfiVariable, UefiResult};

pub const EFI_SECURE_BOOT_ENABLE_DISABLE_GUID: efi::Guid = efi::Guid::from_fields(
    0xf0a30bc7,
    0xaf08,
    0x4556,
    0x99,
    0xc4,
    &[0x0, 0x10, 0x9, 0xc9, 0x3a, 0x44],
);

pub const EFI_IMAGE_SECURITY_DATABASE_GUID: efi::Guid = efi::Guid::from_fields(
    0xd719b2cb,
    0x3d3a,
    0x4596,
    0xa3,
    0xbc,
    &[0xda, 0xd0, 0xe, 0x67, 0x65, 0x6f],
);

pub const EFI_CERT_SHA256_GUID: efi::Guid = efi::Guid::from_fields(
    0xc1c41626,
    0x504c,
    0x4092,
    0xac,
    0xa9,
    &[0x41, 0xf9, 0x36, 0x93, 0x43, 0x28],
);

pub const EFI_CERT_RSA2048_GUID: efi::Guid = efi::Guid::from_fields(
    0x3c5766e8,
    0x269c,
    0x4e34,
    0xaa,
    0x14,
    &[0xed, 0x77, 0x6e, 0x85, 0xb3, 0xb6],
);

pub const EFI_CERT_RSA2048_SHA256_GUID: efi::Guid = efi::Guid::from_fields(
    0xe2b36190,
    0x879b,
    0x4a3d,
    0xad,
    0x8d,
    &[0xf2, 0xe7, 0xbb, 0xa3, 0x27, 0x84],
);

pub const EFI_CERT_SHA1_GUID: efi::Guid = efi::Guid::from_fields(
    0x826ca512,
    0xcf10,
    0x4ac9,
    0xb1,
    0x87,
    &[0xbe, 0x1, 0x49, 0x66, 0x31, 0xbd],
);

pub const EFI_CERT_RSA2048_SHA1_GUID: efi::Guid = efi::Guid::from_fields(
    0x67f8444f,
    0x8743,
    0x48f1,
    0xa3,
    0x28,
    &[0x1e, 0xaa, 0xb8, 0x73, 0x60, 0x80],
);

pub const EFI_CERT_X509_GUID: efi::Guid = efi::Guid::from_fields(
    0xa5c059a1,
    0x94e4,
    0x4aa7,
    0x87,
    0xb5,
    &[0xab, 0x15, 0x5c, 0x2b, 0xf0, 0x72],
);

pub const EFI_CERT_SHA224_GUID: efi::Guid = efi::Guid::from_fields(
    0xb6e5233,
    0xa65c,
    0x44c9,
    0x94,
    0x7,
    &[0xd9, 0xab, 0x83, 0xbf, 0xc8, 0xbd],
);

pub const EFI_CERT_SHA384_GUID: efi::Guid = efi::Guid::from_fields(
    0xff3e5307,
    0x9fd0,
    0x48c9,
    0x85,
    0xf1,
    &[0x8a, 0xd5, 0x6c, 0x70, 0x1e, 0x1],
);

pub const EFI_CERT_SHA512_GUID: efi::Guid = efi::Guid::from_fields(
    0x93e0fae,
    0xa6c4,
    0x4f50,
    0x9f,
    0x1b,
    &[0xd4, 0x1e, 0x2b, 0x89, 0xc1, 0x9a],
);

pub const EFI_CERT_X509_SHA256_GUID: efi::Guid = efi::Guid::from_fields(
    0x3bd2a492,
    0x96c0,
    0x4079,
    0xb4,
    0x20,
    &[0xfc, 0xf9, 0x8e, 0xf1, 0x03, 0xed],
);

pub const EFI_CERT_X509_SHA384_GUID: efi::Guid = efi::Guid::from_fields(
    0x7076876e,
    0x80c2,
    0x4ee6,
    0xaa,
    0xd2,
    &[0x28, 0xb3, 0x49, 0xa6, 0x86, 0x5b],
);

pub const EFI_CERT_X509_SHA512_GUID: efi::Guid = efi::Guid::from_fields(
    0x446dbf63,
    0x2502,
    0x4cda,
    0xbc,
    0xfa,
    &[0x24, 0x65, 0xd2, 0xb0, 0xfe, 0x9d],
);

pub const EFI_CERT_TYPE_PKCS7_GUID: efi::Guid = efi::Guid::from_fields(
    0x4aafd29d,
    0x68df,
    0x49ee,
    0x8a,
    0xa9,
    &[0x34, 0x7d, 0x37, 0x56, 0x65, 0xa7],
);

pub const EFI_IMAGE_SECURITY_DATABASE: &str = "db";
pub const EFI_IMAGE_SECURITY_DATABASE1: &str = "dbx";
pub const EFI_IMAGE_SECURITY_DATABASE2: &str = "dbt";

pub const SECURE_BOOT_MODE_ENABLE: u8 = 1;
pub const SECURE_BOOT_MODE_DISABLE: u8 = 0;

pub const SETUP_MODE: u8 = 1;
pub const USER_MODE: u8 = 0;

#[derive(Debug, Clone)]
pub struct EfiAuthVariable2 {
    pub variable: EfiVariable,
    pub time: efi::Time,
}

impl EfiAuthVariable2 {
    pub fn get_tbs_data_size(&self) -> usize {
        // This is deeply inefficient.
        // A better solution might be to use setters and update an internal size.
        (self.variable.name.chars().count() * core::mem::size_of::<efi::Char16>())
            + core::mem::size_of::<efi::Guid>()
            + core::mem::size_of::<u32>()
            + core::mem::size_of::<efi::Time>()
            + self.variable.data.len()
    }

    pub fn get_tbs_data_buffer(&self, buffer: &mut [u8]) -> UefiResult<usize> {
        // Populate the variable_name.
        let u16_str = OsString::from(self.variable.name.as_str());
        let u16_str_wo_null = &u16_str.as_u16_slice()[..u16_str.len() - 1];
        let (u16_bytes, remainder) =
            buffer.split_at_mut(u16_str_wo_null.len() * core::mem::size_of::<u16>());
        unsafe {
            let slice_start = u16_bytes.as_mut_ptr();
            let u16_slice =
                core::slice::from_raw_parts_mut(slice_start as *mut u16, u16_str_wo_null.len());
            u16_slice.copy_from_slice(u16_str_wo_null);
        }

        // Populate the guid.
        let (guid_bytes, remainder) = remainder.split_at_mut(core::mem::size_of::<efi::Guid>());
        guid_bytes.copy_from_slice(self.variable.guid.as_bytes());

        // Populate the attributes.
        let (attr_bytes, remainder) = remainder.split_at_mut(core::mem::size_of::<u32>());
        unsafe {
            let attr_u32 = attr_bytes.as_mut_ptr();
            *(attr_u32 as *mut u32) = self.variable.attributes;
        }

        // Populate the timestamp.
        let (time_bytes, data_bytes) = remainder.split_at_mut(core::mem::size_of::<efi::Time>());
        unsafe {
            let time_src_bytes = core::ptr::addr_of!(self.time) as *const u8;
            time_bytes.copy_from_slice(core::slice::from_raw_parts(
                time_src_bytes,
                time_bytes.len(),
            ));
        }

        // Populate the data.
        data_bytes.copy_from_slice(&self.variable.data);

        Ok(buffer.len())
    }

    pub fn get_tbs_data(&self) -> UefiResult<Vec<u8>> {
        let tbs_size = self.get_tbs_data_size();
        let mut data = Vec::<u8>::with_capacity(tbs_size);
        // Safe because we're about to init this exact data.
        unsafe { data.set_len(tbs_size) };
        let actual_size = self.get_tbs_data_buffer(&mut data)?;
        unsafe { data.set_len(actual_size) };
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::{efi, EfiAuthVariable2, EfiVariable};

    #[test]
    fn auth_var_2_should_create_digest_to_sign() {
        let test_var = EfiAuthVariable2 {
            variable: EfiVariable {
                name: String::from("TestVar"),
                guid: crate::variable::EFI_GLOBAL_VARIABLE_GUID.clone(),
                data: Vec::<u8>::from([0xDEu8, 0xADu8, 0xBEu8, 0xEFu8]),
                attributes: (efi::VARIABLE_NON_VOLATILE | efi::VARIABLE_BOOTSERVICE_ACCESS),
            },
            time: crate::rustified::Time {
                year: 2022,
                month: 04,
                day: 18,
                ..Default::default()
            }
            .into(),
        };

        assert_eq!(
            test_var.get_tbs_data().unwrap(),
            &[
                0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x56, 0x00, 0x61, 0x00, 0x72, 0x00,
                0x61, 0xDF, 0xE4, 0x8B, 0xCA, 0x93, 0xD2, 0x11, 0xAA, 0x0D, 0x00, 0xE0, 0x98, 0x03,
                0x2B, 0x8C, 0x03, 0x00, 0x00, 0x00, 0xE6, 0x07, 0x04, 0x12, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF
            ]
        );
    }
}
