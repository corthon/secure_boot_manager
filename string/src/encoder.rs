// Copyright (c) 2019 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BufferOverFlow,
    MultiByte,
    NotUtf8String,
    Impossible,
}

/// Encode UTF-8 str to UCS2 with a callback function.
/// F is a function take one parameter Result<u16, Error>
/// param: u16 is each ucs2 encode's character
///
pub fn encode_fnc<F>(input: &str, mut fnc: F) -> Result<usize, Error>
where
    F: FnMut(Result<u16, Error>) -> Result<usize, Error>,
{
    let mut bytes_index = 0;
    let bytes = input.as_bytes();
    let bytes_len = bytes.len();

    while bytes_index < bytes_len {
        let ret;

        // REF： https://tools.ietf.org/html/rfc3629

        match bytes[bytes_index] {
            0b0000_0000..=0b0111_1111 => {
                // 1 byte
                ret = Ok(u16::from(bytes[bytes_index]));
                bytes_index += 1;
            }
            0b1100_0000..=0b1101_1111 => {
                // 2 byte
                if bytes_index + 2 > bytes_len {
                    return Err(Error::NotUtf8String);
                }
                let a = u16::from(bytes[bytes_index] & 0b0001_1111);
                let b = u16::from(bytes[bytes_index + 1] & 0b0011_1111);
                ret = Ok(a << 6 | b);
                bytes_index += 2;
            }
            0b1110_0000..=0b1110_1111 => {
                // 3 byte
                if bytes_index + 3 > bytes_len {
                    return Err(Error::NotUtf8String);
                }
                let a = u16::from(bytes[bytes_index] & 0b0000_1111);
                let b = u16::from(bytes[bytes_index + 1] & 0b0011_1111);
                let c = u16::from(bytes[bytes_index + 2] & 0b0011_1111);
                ret = Ok(a << 12 | b << 6 | c);
                bytes_index += 3;
            }
            0b1111_0000..=0b1111_0111 => {
                // 4 byte
                if bytes_index + 4 > bytes_len {
                    return Err(Error::NotUtf8String);
                }
                ret = Err(Error::MultiByte);
            }
            _ => {
                return Err(Error::NotUtf8String);
            }
        }
        fnc(ret)?;
    }

    Ok(bytes_index)
}

/// Encode UTF-8 str to an u16 array(UCS-2 encode string).
///
/// # Example
///
/// ```rust
/// use efi_str::encoder::*;
/// let mut buffer = [0u16; 1];
/// assert_eq!(encode("中", &mut buffer).is_ok(), true);
/// ```
pub fn encode(input: &str, buffer: &mut [u16]) -> Result<usize, Error> {
    let mut i = 0;
    let buffer_len = buffer.len();
    encode_fnc(input, |ret| {
        match ret {
            Ok(ch) => {
                if i > buffer_len + 1 {
                    return Err(Error::BufferOverFlow);
                }
                buffer[i] = ch;
                i += 1;
            }
            Err(err) => {
                return Err(err);
            }
        }
        Ok(i)
    })?;
    Ok(i)
}

/// Decode an u16 array(UCS2 encode string) to an u8 array(UTF8 encode string) .
///
/// # Example
///
/// ```rust
/// use efi_str::encoder::*;
/// let mut u8_buffer = [0u8; 6];
/// let u16_str = [0x4e2du16, 0x56fdu16];
/// let len = decode(&u16_str, &mut u8_buffer).unwrap_or(0);
/// assert_eq!(len, 6);
/// assert_eq!(core::str::from_utf8(&u8_buffer[..]), Ok("中国"));
/// ```
pub fn decode(input: &[u16], buffer: &mut [u8]) -> Result<usize, Error> {
    let buffer_size = buffer.len();
    let mut index = 0;

    for &ch in input.iter() {
        match ch {
            0x0000..=0x007F => {
                // 1 byte
                if index + 1 > buffer_size {
                    return Err(Error::BufferOverFlow);
                }
                buffer[index] = ch as u8;
                index += 1;
            }
            0x0080..=0x07FF => {
                // 2 byte
                if index + 2 > buffer_size {
                    return Err(Error::BufferOverFlow);
                }
                let ch0_6 = ((ch << 10) >> 10) as u8;
                let ch6_12 = ((ch << 5) >> 11) as u8;
                //let ch12_16 = ((ch << 0) >> 12) as u8;
                buffer[index] = 0b1100_0000 + ch6_12 as u8;
                buffer[index + 1] = 0b1000_0000 + ch0_6 as u8;
            }
            0x800..=0xFFFF => {
                // 3 byte
                if index + 3 > buffer_size {
                    return Err(Error::BufferOverFlow);
                }
                let ch0_6 = ((ch << 10) >> 10) as u8;
                let ch6_12 = ((ch << 4) >> (4 + 6)) as u8;
                let ch12_16 = (ch >> 12) as u8;
                buffer[index] = 0b1110_0000 + ch12_16;
                buffer[index + 1] = 0b1000_0000 + ch6_12;
                buffer[index + 2] = 0b1000_0000 + ch0_6;
                index += 3;
            }
        }
    }

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let input = "中国";
        let mut buffer = [0u16; 2];
        assert_eq!(encode(input, &mut buffer).is_ok(), true);
        assert_eq!(buffer[0], 0x4e2du16);
        assert_eq!(buffer[1], 0x56fdu16);
    }

    #[test]
    fn test_decode() {
        let mut u8_buffer = [0u8; 6];
        let u16_str = [0x4e2du16, 0x56fdu16]; // 中国
        let len = decode(&u16_str, &mut u8_buffer).unwrap_or(0);
        assert_eq!(len, 6);
        assert_eq!(core::str::from_utf8(&u8_buffer[..]), Ok("中国"));
    }
}
