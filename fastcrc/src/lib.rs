//! FastCRC provides CRC implementations under a single crate.
//!
//! The crate is currently structured as two layers:
//! - [`core`] hosts reusable engines, parameter descriptions, and future shared utilities.
//! - [`crc32`] contains concrete CRC32 variants plus ergonomic helpers.
//!
//! Top-level re-exports make the most common types available directly from the
//! crate root for convenience.

#![deny(unsafe_code)]

mod core;
mod crc32;
mod crc32c;

pub use crate::crc32::{crc32, Crc32};
pub use crate::crc32c::{crc32c, Crc32c};
