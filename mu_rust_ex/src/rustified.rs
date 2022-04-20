/* @file
Rustified wrappers/extensions for existing EFI C structures.
These make it nicer to work with these basic structures while
maintaining From compatibility to/from the raw structures.

Copyright (c) Microsoft Corporation.
SPDX-License-Identifier: BSD-2-Clause-Patent

*/

use core::convert::TryFrom;

use r_efi::efi;
use r_efi::efi::{TIME_ADJUST_DAYLIGHT, TIME_IN_DAYLIGHT};

#[derive(Debug, Clone, Copy, Default)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    // pub pad1: u8,
    pub nanosecond: u32,
    pub timezone: i16,
    pub daylight: u8,
    // pub pad2: u8,
}

impl From<Time> for efi::Time {
    fn from(f: Time) -> Self {
        Self {
            year: f.year,
            month: f.month,
            day: f.day,
            hour: f.hour,
            minute: f.minute,
            second: f.second,
            nanosecond: f.nanosecond,
            timezone: f.timezone,
            daylight: f.daylight,
            pad1: 0,
            pad2: 0,
        }
    }
}

impl TryFrom<efi::Time> for Time {
    type Error = efi::Status;
    fn try_from(tf: efi::Time) -> Result<Self, Self::Error> {
        let tad = (tf.daylight & TIME_ADJUST_DAYLIGHT) == TIME_ADJUST_DAYLIGHT;
        let tid = (tf.daylight & TIME_IN_DAYLIGHT) == TIME_IN_DAYLIGHT;
        if ((tf.daylight & 0xFC) != 0) || (tid && !tad) {
            return Err(efi::Status::INVALID_PARAMETER);
        }

        Ok(Self {
            year: tf.year,
            month: tf.month,
            day: tf.day,
            hour: tf.hour,
            minute: tf.minute,
            second: tf.second,
            nanosecond: tf.nanosecond,
            timezone: tf.timezone,
            daylight: tf.daylight,
        })
    }
}
