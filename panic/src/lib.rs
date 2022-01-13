// Copyright (c) 2019 Intel Corporation
// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]
#![feature(lang_items)]
#![allow(unused)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
fn eh_personality() {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_lib() {}
}
