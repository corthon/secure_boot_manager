# Secure Boot Manager

This is a learning project -- written in Rust -- to be a full Shell-based management
utility for UEFI Secure Boot.

## High-Level Goals

The following features should be achievable (with some stretch goals == ?):

- Let user walk PCI tree and identify OpROMs to "bless"
- Walk mounted file systems and identify apps/bootloaders to "bless"
- Take ownership of PK if in SetupMode and populate KEK
- Keep two databases of "human readible" strings for printing details
  - One built-in that can be updated with "known" values
  - One that the user can maintain with strings as they add new values
- Use various methods to sign updates to SB variables
  - OpenSSL cert with password support
  - Yubikey?
  - TPM?
- Allow user to install "common" KEK and db values
  - UEFI Cert
  - RedHat Cert
  - MS Cert
  - MS KEK signer
  - Linux registry KEK signer

## Low-Level TODOs

- Parse SigLists
- Figure out how to sign
  - EDK2 SharedCrypto?
- Wrapper for walking PCI
- Wrapper for walking FSes
- Way to request a ConnectAll?
- Way to request "map -r"?

## Investigate

- https://docs.rs/menu/0.3.2/menu/#menu

- https://docs.rs/asn1_der/latest/asn1_der/
- https://docs.rs/x509-parser/0.6.0/x509_parser/

- https://docs.rs/serde-json-core/latest/serde_json_core/
- https://docs.rs/heapless/0.7.9/heapless/pool/struct.Pool.html

- https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/ffi.html
- http://jakegoulding.com/rust-ffi-omnibus/objects/

- https://docs.rs/ring/latest/ring/signature/index.html
- https://lib.rs/crates/ring
- https://docs.rs/pkcs1/latest/pkcs1/struct.RsaPrivateKey.html#method.to_der

- https://docs.microsoft.com/en-us/azure/iot-hub/tutorial-x509-self-sign

- https://docs.rs/cbindgen/latest/cbindgen/

## Build Command

```bash
cargo +nightly build -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem --target x86_64-unknown-uefi --manifest-path C:\_uefi\mu_ci\mu_tiano_platforms\SBManage\Cargo.toml
```