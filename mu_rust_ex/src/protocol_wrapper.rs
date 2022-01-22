use r_efi::efi;

pub trait ManagedProtocol {
    type ProtocolType;

    fn get_guid() -> efi::Guid;
    fn get_handle() -> efi::Handle;
    fn init_protocol(&mut self, prot: *mut core::ffi::c_void, hand: efi::Handle) -> Result<Self::ProtocolType, efi::Status>;
    fn deinit_protocol(&mut self);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ManagedProtocolError {
    Unregistered,
    Efi(efi::Status),
}

pub struct ProtocolWrapper<T: ManagedProtocol> {
    inner: T,
}

impl<T: ManagedProtocol> ProtocolWrapper<T> {

}

// TODO: impl Deref for ProtocolWrapper
// TODO: impl DerefMut for ProtocolWrapper
// TODO: impl Drop for ProtocolWrapper

// THEN...
// The struct that implements ManagedProtocol will be responsible for returning special result enums for when the protocol was yanked.
