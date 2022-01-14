// Copyright (c) 2019 Intel Corporation
// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent
#![no_std]

use core_con_out::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {} // TODO: Try to replace with an exit of sorts.
}
