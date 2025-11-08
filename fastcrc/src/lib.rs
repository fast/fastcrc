// Copyright 2024 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! FastCRC provides CRC implementations under a single crate.
//!
//! The crate is structured in layers: [`core`] hosts reusable engines and
//! algorithm descriptions, while algorithm-specific modules such as [`crc32`]
//! and [`crc32c`] expose ergonomic digest types. The crate root re-exports the
//! most common types so users can depend on `fastcrc` alone.

#![deny(unsafe_code)]

mod core;
mod crc32;
mod crc32c;

pub use crate::crc32::{crc32, Crc32};
pub use crate::crc32c::{crc32c, Crc32c};
