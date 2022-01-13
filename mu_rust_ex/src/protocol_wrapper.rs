trait ManagedProtocol {
  pub const fn get_guid() -> efi::Guid;
  pub fn init_protocol(&mut self, prot: *mut core::ffi::c_void) -> Result<(), ()>;
  pub fn deinit_protocol(&mut self);
}

// struct ProtocolWrapper<T where T: impl ManagedProtocol>;  // Or something.
// Implements Deref and DerefMut
// Has the registration and any metadata to figure out how to: 1) deinit "self" and 2) when dropped, unregister this handle.

// THEN...
// The struct that implements ManagedProtocol will be responsible for returning special result enums for when the protocol was yanked.
