// Copyright (c) 2019 Intel Corporation
// Copyright (c) Microsoft Corporation.
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![feature(alloc_error_handler)]
#![cfg_attr(not(test), no_std)]

use core::ptr::NonNull;

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use r_efi::efi;
use spin::Mutex;

const UNINIT_FAIL_STR: &str = "uefi allocator not initialized";

pub struct UnsafeUefiPtr<T>(NonNull<T>);

// [unsafe] UEFI structures are, by nature, not thread-safe.
//          In the strictest of sense, this means that these pointers
//          can never be shared between threads. However, since the rest
//          of the system is also single-threaded, we can make some
//          assumptions around whether this is handled correctly (presuming
//          the underlying implemnetation is sound).
unsafe impl<T> Sync for UnsafeUefiPtr<T> {}
unsafe impl<T> Send for UnsafeUefiPtr<T> {}

impl<T> AsMut<T> for UnsafeUefiPtr<T> {
    fn as_mut(&mut self) -> &mut T {
        // [unsafe] Requires that the pointer be initialized
        //          with a legitimate UEFI structure.
        unsafe { self.0.as_mut() }
    }
}

pub struct BsAllocator(Mutex<Option<UnsafeUefiPtr<efi::BootServices>>>);
impl BsAllocator {
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }

    // [unsafe] Caller must ensure that `st` is a valid efi::SystemTable
    //          instance that is correctly aligned for the architecture.
    pub unsafe fn init(&self, st: *mut efi::SystemTable) -> Result<(), efi::Status> {
        let bs_ptr = NonNull::new((*st).boot_services).map(|ptr| UnsafeUefiPtr(ptr));
        let mut guard = self.0.lock();
        match *guard {
            Some(_) => Err(efi::Status::ALREADY_STARTED),
            None => {
                *guard = bs_ptr;
                Ok(())
            }
        }
    }
}

// [unsafe] Because the Trait requires it. Relies on the allocator
//          making wise decisions.
unsafe impl GlobalAlloc for BsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        if align > 8 {
            return core::ptr::null_mut();
        }

        let mut guard = self.0.lock();
        let bs_mut = guard.as_mut().expect(UNINIT_FAIL_STR);
        let mut address: *mut c_void = core::ptr::null_mut();

        match (bs_mut.as_mut().allocate_pool)(
            efi::BOOT_SERVICES_DATA,
            size,
            &mut address as *mut *mut _,
        ) {
            efi::Status::SUCCESS => address as *mut u8,
            _ => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let mut guard = self.0.lock();
        let bs_mut = guard.as_mut().expect(UNINIT_FAIL_STR);
        match (bs_mut.as_mut().free_pool)(ptr as *mut _) {
            efi::Status::SUCCESS => (),
            e => panic!("failure during uefi dealloc {:?}", e),
        }
    }
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: BsAllocator = BsAllocator::new();

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    panic!("uefi allocator failure")
}

// [unsafe] Caller must ensure that `st` is a valid efi::SystemTable
//          instance that is correctly aligned for the architecture.
#[cfg(not(test))]
pub unsafe fn init(st: *mut efi::SystemTable) -> Result<(), efi::Status> {
    ALLOCATOR.init(st)
}
