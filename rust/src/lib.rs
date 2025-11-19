// Re-export iqrah-api as the public interface
// This maintains compatibility with existing Flutter bridge code
extern crate iqrah_api;
pub use iqrah_api::*;

// Flutter Rust Bridge generated code
mod frb_generated;
