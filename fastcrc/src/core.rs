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

use core::fmt;

use digest::core_api::OutputSizeUser;
use digest::typenum::U4;
use digest::{FixedOutput, FixedOutputReset, Output, Reset, Update};

/// Describes a CRC32 variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Algorithm32 {
    /// Human friendly name (used for debug output or registry keys).
    pub name: &'static str,
    /// Standard (non-reflected) polynomial without the top bit.
    pub polynomial: u32,
    /// Initial register value.
    pub init: u32,
    /// Final XOR mask applied after the optional reflection step.
    pub xor_out: u32,
    /// Whether input bytes are processed in reflected form.
    pub reflect_in: bool,
    /// Whether the final CRC value is reflected before `xor_out` is applied.
    pub reflect_out: bool,
}

impl Algorithm32 {
    /// Construct a new CRC32 algorithm description.
    pub const fn new(
        name: &'static str,
        polynomial: u32,
        init: u32,
        xor_out: u32,
        reflect_in: bool,
        reflect_out: bool,
    ) -> Self {
        Self {
            name,
            polynomial,
            init,
            xor_out,
            reflect_in,
            reflect_out,
        }
    }
}

/// Streaming CRC32 engine that can host any [`Algorithm32`].
#[derive(Clone)]
pub(crate) struct Crc32Engine {
    params: Algorithm32,
    table: [u32; 256],
    state: u32,
}

impl Crc32Engine {
    /// Build a new CRC32 engine for the provided algorithm description.
    pub(crate) fn new(params: Algorithm32) -> Self {
        Self {
            params,
            table: build_table(params.polynomial, params.reflect_in),
            state: params.init,
        }
    }

    fn absorb(&mut self, data: &[u8]) {
        self.state = if self.params.reflect_in {
            update_reflected(self.state, &self.table, data)
        } else {
            update_standard(self.state, &self.table, data)
        };
    }

    /// Update the digest state with additional bytes.
    pub(crate) fn update(&mut self, data: &[u8]) {
        self.absorb(data);
    }

    /// Reset the digest to its initial value.
    pub(crate) fn reset(&mut self) {
        self.state = self.params.init;
    }

    /// Retrieve the finalized checksum as `u32`.
    pub(crate) fn finalize_u32(&self) -> u32 {
        finalize_value(self.state, self.params)
    }
}

impl fmt::Debug for Crc32Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Crc32Engine")
            .field("algorithm", &self.params.name)
            .field("state", &format_args!("0x{state:08x}", state = self.state))
            .finish()
    }
}

impl OutputSizeUser for Crc32Engine {
    type OutputSize = U4;
}

impl Update for Crc32Engine {
    fn update(&mut self, data: &[u8]) {
        self.absorb(data);
    }
}

impl FixedOutput for Crc32Engine {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(&self.finalize_u32().to_be_bytes());
    }
}

impl FixedOutputReset for Crc32Engine {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        out.copy_from_slice(&self.finalize_u32().to_be_bytes());
        self.reset();
    }
}

impl Reset for Crc32Engine {
    fn reset(&mut self) {
        Crc32Engine::reset(self);
    }
}

fn finalize_value(state: u32, params: Algorithm32) -> u32 {
    let mut crc = state;
    if params.reflect_in ^ params.reflect_out {
        crc = reflect_bits(crc, 32);
    }
    crc ^ params.xor_out
}

fn update_reflected(mut state: u32, table: &[u32; 256], data: &[u8]) -> u32 {
    for &byte in data {
        let idx = ((state as u8) ^ byte) as usize;
        state = (state >> 8) ^ table[idx];
    }
    state
}

fn update_standard(mut state: u32, table: &[u32; 256], data: &[u8]) -> u32 {
    for &byte in data {
        let idx = (((state >> 24) as u8) ^ byte) as usize;
        state = (state << 8) ^ table[idx];
    }
    state
}

fn build_table(polynomial: u32, reflect: bool) -> [u32; 256] {
    let mut table = [0u32; 256];
    if reflect {
        let reflected = reflect_bits(polynomial, 32);
        for (i, slot) in table.iter_mut().enumerate() {
            let mut crc = i as u32;
            for _ in 0..8 {
                if (crc & 1) != 0 {
                    crc = (crc >> 1) ^ reflected;
                } else {
                    crc >>= 1;
                }
            }
            *slot = crc;
        }
    } else {
        for (i, slot) in table.iter_mut().enumerate() {
            let mut crc = (i as u32) << 24;
            for _ in 0..8 {
                if (crc & 0x8000_0000) != 0 {
                    crc = (crc << 1) ^ polynomial;
                } else {
                    crc <<= 1;
                }
            }
            *slot = crc;
        }
    }
    table
}

fn reflect_bits(mut value: u32, width: u8) -> u32 {
    let mut reversed = 0u32;
    let mut i = 0;
    while i < width {
        reversed <<= 1;
        if (value & 1) != 0 {
            reversed |= 1;
        }
        value >>= 1;
        i += 1;
    }
    reversed
}

#[cfg(test)]
mod tests {
    use super::*;

    const IEEE: Algorithm32 =
        Algorithm32::new("crc32", 0x04C11DB7, 0xFFFF_FFFF, 0xFFFF_FFFF, true, true);

    #[test]
    fn reflect_roundtrip() {
        assert_eq!(reflect_bits(0b1001, 4), 0b1001);
        assert_eq!(reflect_bits(0b0011, 4), 0b1100);
    }

    #[test]
    fn dyn_engine_matches_known_checksum() {
        let mut engine = Crc32Engine::new(IEEE);
        engine.update(b"123456789");
        assert_eq!(engine.finalize_u32(), 0xCBF4_3926);
    }
}
