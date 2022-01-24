use core::ops::{Deref, DerefMut};

use r_efi::efi;

use crate::{boot, UefiResult};

pub trait ManagedProtocol {
    type ProtocolType: ManagedProtocol;

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

pub struct ProtocolWrapper<T> {
    inner: T,
}

impl<T: ManagedProtocol<ProtocolType = T>> ProtocolWrapper<T> {
    // TODO: Update UEFI to return the handle in LocateProtocol (or some special LocateProtocol).
    pub fn first() -> UefiResult<Self> {
        let bs = boot::uefi_bs();
        let prot_handles = bs.locate_protocol_handles(T::get_guid())?;
        let handle = prot_handles[0];

        let prot_ptr = bs.get_protocol(T::get_guid(), handle)?;
        Ok(Self { inner: T::init_protocol(prot_ptr, handle)? })
    }
}
impl<T> Deref for ProtocolWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for ProtocolWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// TODO: impl Drop for ProtocolWrapper

// THEN...
// The struct that implements ManagedProtocol will be responsible for returning special result enums for when the protocol was yanked.
