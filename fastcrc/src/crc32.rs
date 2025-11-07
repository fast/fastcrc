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

use digest::core_api::OutputSizeUser;
use digest::typenum::U4;
use digest::{FixedOutput, FixedOutputReset, HashMarker, Output, Reset, Update};

use crate::core::{Algorithm32, Crc32Engine};

/// Classic Ethernet CRC32 (a.k.a. IEEE, ISO-HDLC).
pub(crate) const CRC32: Algorithm32 =
    Algorithm32::new("crc32", 0x04C11DB7, 0xFFFF_FFFF, 0xFFFF_FFFF, true, true);

/// CRC32 digest implementing the RustCrypto [`digest::Digest`] blanket impl.
#[derive(Clone)]
pub struct Crc32 {
    inner: Crc32Engine,
}

impl Crc32 {
    /// Create a new CRC32 (IEEE) digest instance.
    pub fn new() -> Self {
        Self {
            inner: Crc32Engine::new(CRC32),
        }
    }

    /// Retrieve the checksum as `u32`.
    pub fn finalize_u32(self) -> u32 {
        self.inner.finalize_u32()
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputSizeUser for Crc32 {
    type OutputSize = U4;
}

impl Update for Crc32 {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }
}

impl Reset for Crc32 {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl FixedOutput for Crc32 {
    fn finalize_into(self, out: &mut Output<Self>) {
        self.inner.finalize_into(out);
    }
}

impl FixedOutputReset for Crc32 {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        self.inner.finalize_into_reset(out);
    }
}

impl HashMarker for Crc32 {}

/// One-shot helper for calculating IEEE CRC32 over a byte slice.
pub fn crc32(data: &[u8]) -> u32 {
    let mut digest = Crc32::new();
    digest.update(data);
    digest.finalize_u32()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_value() {
        let checksum = crc32(b"123456789");
        assert_eq!(checksum, 0xCBF4_3926);
    }
}
