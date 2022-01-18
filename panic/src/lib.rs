// Copyright (c) 2019 Intel Corporation
// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent
#![no_std]

use core_con_out::print_panic;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { print_panic(format_args!("{}\r\n", info)) };
    loop {} // TODO: Try to replace with an exit of sorts.
}
