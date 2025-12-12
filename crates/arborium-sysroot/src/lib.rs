// This crate provides the wasm-sysroot path to dependent crates
// via the DEP_ARBORIUM_SYSROOT_PATH environment variable set by build.rs,
// and includes WASM allocator implementations for browser compatibility.

// Include the WASM allocator module when targeting WASM
#[cfg(target_family = "wasm")]
mod wasm;

// Re-export allocator symbols for external crates
#[cfg(target_family = "wasm")]
pub use wasm::*;
