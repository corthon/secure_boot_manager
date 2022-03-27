use alloc::sync::Arc;

use r_efi::efi;
use spin::Mutex;

use crate::{boot, UefiResult};

mod ptr {
    use core::cmp::{Ord, PartialOrd};
    use core::convert::TryFrom;
    use core::ffi::c_void;
    use core::ptr::NonNull;

    use super::efi;

    // TODO: Add a bunch of notes about why these types are here, how UEFI BS is
    //       already unsafe, so we really can't guarantee some of these principles anyway,
    //       and how we can know which uses are "safe".

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct EfiOpaqueHandle(usize);
    impl From<efi::Handle> for EfiOpaqueHandle {
        fn from(f: efi::Handle) -> Self {
            Self(f as usize)
        }
    }
    impl From<EfiOpaqueHandle> for efi::Handle {
        fn from(f: EfiOpaqueHandle) -> Self {
            f.0 as Self
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct EfiProtocolPtr<T>(NonNull<T>);
    pub type AnyProtocol = c_void;
    impl<T> EfiProtocolPtr<T> {
        pub fn as_ref(&self) -> &T {
            unsafe { self.0.as_ref() }
        }
        pub fn as_mut(&mut self) -> &mut T {
            unsafe { self.0.as_mut() }
        }
    }
    impl<T> From<EfiProtocolPtr<T>> for *mut T {
        fn from(f: EfiProtocolPtr<T>) -> Self {
            f.0.as_ptr()
        }
    }
    impl<T> TryFrom<*mut T> for EfiProtocolPtr<T> {
        type Error = ();
        fn try_from(f: *mut T) -> Result<Self, Self::Error> {
            Ok(Self(NonNull::new(f).ok_or(())?))
        }
    }

    unsafe impl<T> Send for EfiProtocolPtr<T> {}
    unsafe impl<T> Sync for EfiProtocolPtr<T> {}
}
pub use ptr::{AnyProtocol, EfiOpaqueHandle, EfiProtocolPtr};

mod manager {
    use alloc::collections::BTreeMap;
    use alloc::sync::{Arc, Weak};
    use core::cmp::{Ord, Ordering, PartialOrd};
    use core::convert::TryFrom;
    use core::ops::{Deref, DerefMut};

    use super::{boot, efi, Mutex};
    use super::{AnyProtocol, EfiOpaqueHandle, EfiProtocolPtr};

    #[derive(Debug, PartialEq, Eq)]
    struct ProtocolCacheKey {
        guid: efi::Guid,
        handle: EfiOpaqueHandle,
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

    pub struct ManagedProtocol<T> {
        ptr: EfiProtocolPtr<T>,
        guid: efi::Guid,
        handle: EfiOpaqueHandle,
    }
    impl<T> Deref for ManagedProtocol<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.ptr.as_ref()
        }
    }
    impl<T> DerefMut for ManagedProtocol<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ptr.as_mut()
        }
    }
    // TODO: Implement Drop for this.
    // https://doc.rust-lang.org/std/sync/struct.Arc.html#method.strong_count

    type CacheEntry = Arc<Mutex<Option<ManagedProtocol<AnyProtocol>>>>;
    type InternalCacheEntry = Weak<Mutex<Option<ManagedProtocol<AnyProtocol>>>>;
    lazy_static! {
        static ref PROTOCOL_CACHE: Mutex<BTreeMap<ProtocolCacheKey, InternalCacheEntry>> =
            Mutex::new(BTreeMap::new());
    }

    fn get_cached_instance(handle: efi::Handle, guid: &efi::Guid) -> Option<CacheEntry> {
        let key = ProtocolCacheKey {
            guid: *guid,
            handle: handle.into(),
        };

        PROTOCOL_CACHE
            .lock()
            .get(&key)?
            .upgrade()
            .or_else(|| {
                PROTOCOL_CACHE.lock().remove(&key);
                None
            })
    }

    pub fn find_or_init_cached_instance(
        handle: efi::Handle,
        guid: &efi::Guid,
    ) -> Option<CacheEntry> {
        get_cached_instance(handle, guid).or_else(|| {
            let bs = boot::uefi_bs();
            let prot_ptr = bs.get_protocol(guid, handle).ok()?;
            let cache_entry: CacheEntry = Arc::new(Mutex::new(Some(ManagedProtocol {
                ptr: EfiProtocolPtr::try_from(prot_ptr).unwrap(),
                guid: *guid,
                handle: handle.into(),
            })));
            let key = ProtocolCacheKey {
                guid: *guid,
                handle: handle.into(),
            };

            PROTOCOL_CACHE
                .lock()
                .insert(key, Arc::downgrade(&cache_entry));

            Some(cache_entry)
        })
    }
}
pub use manager::ManagedProtocol;

#[derive(Debug, Copy, Clone)]
pub enum RustProtocolError {
    Unregistered,
    Efi(efi::Status),
}
impl From<RustProtocolError> for efi::Status {
    fn from(f: RustProtocolError) -> Self {
        match f {
            RustProtocolError::Unregistered => efi::Status::MEDIA_CHANGED,
            RustProtocolError::Efi(x) => x,
        }
    }
}
impl From<efi::Status> for RustProtocolError {
    fn from(f: efi::Status) -> Self {
        Self::Efi(f)
    }
}
pub type RustProtocolResult<T> = Result<T, RustProtocolError>;

pub trait RustProtocol: Sized {
    type RawProtocol;
    fn get_name() -> &'static str;
    fn get_guid() -> &'static efi::Guid;
    fn init_protocol(
        mp: Arc<Mutex<Option<ManagedProtocol<Self::RawProtocol>>>>,
    ) -> UefiResult<Self>;

    fn first() -> UefiResult<Self> {
        let bs = boot::uefi_bs();
        let prot_handles = bs.locate_protocol_handles(Self::get_guid())?;
        let handle = prot_handles[0];

        Self::by_handle(handle)
    }

    fn by_handle(handle: efi::Handle) -> UefiResult<Self> {
        type ArcMutOpManProt<T> = Arc<Mutex<Option<ManagedProtocol<T>>>>;

        let arc_mp_any = manager::find_or_init_cached_instance(handle, Self::get_guid())
            .ok_or(efi::Status::NOT_FOUND)?;
        // Why is this safe?
        // Well... either we originally found a matching protocol, or not.
        // If there were ever some disconnect, this would be almost impossible to figure out.
        let arc_mp = unsafe {
            core::mem::transmute::<ArcMutOpManProt<AnyProtocol>, ArcMutOpManProt<Self::RawProtocol>>(
                arc_mp_any,
            )
        };
        Self::init_protocol(arc_mp)
    }
}
