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

- Write a Rust wrapper for files
- Parse SigLists
- Figure out how to sign
  - EDK2 SharedCrypto?
- Figure out a better way to interact with the user
  - ConIn/ConOut wrapper? Ncurses?
- Wrapper for walking PCI
- Wrapper for walking FSes
- Way to request a ConnectAll?
- Way to request "map -r"?
