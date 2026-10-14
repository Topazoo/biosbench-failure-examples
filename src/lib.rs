//! Host-only fault examples. No firmware access or system integration.
//!
//! The production-derived save routine is compiled only by the in-memory
//! integration harness. Its UEFI-facing adapter is deliberately not a library API.

pub mod one_use;
pub mod session;
