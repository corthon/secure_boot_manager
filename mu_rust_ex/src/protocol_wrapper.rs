use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use core::cmp::{Ord, Ordering, PartialOrd};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use r_efi::efi;
use spin::Mutex;

use crate::{boot, println, UefiResult};

pub trait ManagedProtocol {
    type ProtocolType: ManagedProtocol;

    fn get_name() -> &'static str;
    fn get_guid() -> &'static efi::Guid;
    fn init_protocol(
        prot: *mut core::ffi::c_void,
        hand: efi::Handle,
    ) -> UefiResult<Self::ProtocolType>;

    fn get_handle(&self) -> efi::Handle;
    fn deinit_protocol(&mut self);
}

#[derive(Debug, Copy, Clone)]
pub enum ManagedProtocolError {
    Unregistered,
    Efi(efi::Status),
}
impl From<ManagedProtocolError> for efi::Status {
    fn from(f: ManagedProtocolError) -> Self {
        match f {
            ManagedProtocolError::Unregistered => efi::Status::MEDIA_CHANGED,
            ManagedProtocolError::Efi(x) => x,
        }
    }
}
pub type ManagedProtocolResult<T> = Result<T, ManagedProtocolError>;

#[derive(Debug, PartialEq, Eq)]
struct ProtocolCacheKey {
    guid: efi::Guid,
    handle: efi::Handle,
}
impl PartialOrd for ProtocolCacheKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ProtocolCacheKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.guid.as_bytes().cmp(&other.guid.as_bytes()) {
            Ordering::Equal => self.handle.cmp(&other.handle),
            a => a,
        }
    }
}
// NOTE: While we're not caring about multithreadedness, this is
//      all well and good. As soon as we care about threading, this table ALSO
//      needs to be wrapped in a Mutex.
static mut CACHED_INSTANCES: BTreeMap<ProtocolCacheKey, NonNull<core::ffi::c_void>> =
    BTreeMap::new();

pub struct ProtocolWrapper<T: ManagedProtocol> {
    inner: Arc<Mutex<T>>,
}

impl<T: ManagedProtocol<ProtocolType = T>> ProtocolWrapper<T> {
    fn get_cached_instance(handle: efi::Handle) -> Option<Arc<Mutex<T>>> {
        let key = ProtocolCacheKey {
            guid: *T::get_guid(),
            handle,
        };
        // TODO: Add a note about why this is safe-ish.
        // TODO: Add a note to ALL unsafes about what the expectations are and
        //       why it should be safe.
        let weak_ref = unsafe {
            let mutex_ptr = CACHED_INSTANCES
                .get(&key)
                .map(|nn_ref| (*nn_ref).as_ptr() as *const Mutex<T>)?;
            Weak::from_raw(mutex_ptr)
        };
        let strong_ref = weak_ref.upgrade();
        if strong_ref.is_some() {
            // If successful, leak the weak ref to prevent dropping.
            let _ = weak_ref.into_raw();
        } else {
            // Otherwise, drop the key from the cache.
            let _ = unsafe { CACHED_INSTANCES.remove(&key) };
            // TODO: If we drop this, make sure we don't need to unregister with BootServices.
        }

        strong_ref
    }

    fn find_or_init_cached_instance(handle: efi::Handle) -> Option<Arc<Mutex<T>>> {
        Self::get_cached_instance(handle).or_else(|| {
            let bs = boot::uefi_bs();
            let prot_ptr = bs.get_protocol(T::get_guid(), handle);

            // TODO: Return an option containing the Arc.
            None
        })
    }

    // TODO: Update UEFI to return the handle in LocateProtocol (or some special LocateProtocol).
    pub fn first() -> UefiResult<Self> {
        let bs = boot::uefi_bs();
        let prot_handles = bs.locate_protocol_handles(T::get_guid())?;
        let handle = prot_handles[0];

        let prot_ptr = bs.get_protocol(T::get_guid(), handle)?;
        Ok(Self {
            inner: T::init_protocol(prot_ptr, handle)?,
        })
    }

    pub fn by_handle(handle: efi::Handle) -> UefiResult<Self> {
        let bs = boot::uefi_bs();

        let prot_ptr = bs.get_protocol(T::get_guid(), handle)?;
        Ok(Self {
            inner: T::init_protocol(prot_ptr, handle)?,
        })
    }
}
impl<T: ManagedProtocol> Deref for ProtocolWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // TODO: This may be bad.
        &*self.inner.lock()
    }
}
impl<T: ManagedProtocol> DerefMut for ProtocolWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: This may be bad.
        &mut *self.inner.lock()
    }
}
impl<T: ManagedProtocol> Drop for ProtocolWrapper<T> {
    fn drop(&mut self) {
        // If the last two references are this one
        // and the one in the cache... then drop the cache.
        if self.inner.strong_count() <= 2 {
            // TODO: Deregister with BootServices
            // TODO: Drop Rc from the cache
        }
        println!("dropping ProtocolWrapper<{}>", T::get_name());
        // TODO: Tell BootServices that we no longer need the deinit callback.
    }
}
