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

/// Castagnoli CRC32 (CRC32C) widely used by SSE4.2 instructions, NVMe, etc.
pub(crate) const CRC32C: Algorithm32 =
    Algorithm32::new("crc32c", 0x1EDC6F41, 0xFFFF_FFFF, 0xFFFF_FFFF, true, true);

/// CRC32C digest implementing the RustCrypto [`digest::Digest`] blanket impl.
#[derive(Clone)]
pub struct Crc32c {
    inner: Crc32Engine,
}

impl Crc32c {
    /// Create a new CRC32C digest instance.
    pub fn new() -> Self {
        Self {
            inner: Crc32Engine::new(CRC32C),
        }
    }

    /// Retrieve the checksum as `u32`.
    pub fn finalize_u32(self) -> u32 {
        self.inner.finalize_u32()
    }
}

impl Default for Crc32c {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputSizeUser for Crc32c {
    type OutputSize = U4;
}

impl Update for Crc32c {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }
}

impl Reset for Crc32c {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl FixedOutput for Crc32c {
    fn finalize_into(self, out: &mut Output<Self>) {
        self.inner.finalize_into(out);
    }
}

impl FixedOutputReset for Crc32c {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        self.inner.finalize_into_reset(out);
    }
}

impl HashMarker for Crc32c {}

/// One-shot helper for calculating Castagnoli CRC32 over a byte slice.
pub fn crc32c(data: &[u8]) -> u32 {
    let mut digest = Crc32c::new();
    digest.update(data);
    digest.finalize_u32()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vectors derived from the RevEng CRC Catalogue
    /// (http://reveng.sourceforge.net/crc-catalogue/), released into the
    /// public domain.
    const CASTAGNOLI_VECTORS: &[(&[u8], u32)] = &[
        (b"", 0x0000_0000),
        (b"123456789", 0xE306_9283),
        (b"The quick brown fox jumps over the lazy dog", 0x2262_0404),
    ];

    #[test]
    fn crc32c_known_value() {
        for &(input, expected) in CASTAGNOLI_VECTORS {
            assert_eq!(crc32c(input), expected, "input: {:?}", input);
        }
    }

    #[test]
    fn crc32c_streaming_matches_one_shot() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let mut digest = Crc32c::new();
        for chunk in data.chunks(7) {
            digest.update(chunk);
        }
        assert_eq!(digest.finalize_u32(), crc32c(data));
    }
}
