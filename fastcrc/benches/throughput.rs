use std::fmt;
use std::sync::OnceLock;

use digest::Digest;
use divan::{black_box, counter::BytesCount, Bencher};
use fastcrc::{Crc32, Crc32c};

fn main() {
    divan::main();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DatasetSpec {
    label: &'static str,
    size: usize,
}

impl DatasetSpec {
    const fn new(label: &'static str, size: usize) -> Self {
        Self { label, size }
    }
}

impl fmt::Display for DatasetSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

const DATASET_SPEC_LIST: [DatasetSpec; 3] = [
    DatasetSpec::new(" 64B", 64),
    DatasetSpec::new("4KiB", 4 * 1024),
    DatasetSpec::new("4MiB", 4 * 1024 * 1024),
];

const DATASET_SPECS: &[DatasetSpec] = &DATASET_SPEC_LIST;

struct DatasetStorage {
    spec: DatasetSpec,
    data: OnceLock<Box<[u8]>>,
}

impl DatasetStorage {
    const fn new(spec: DatasetSpec) -> Self {
        Self {
            spec,
            data: OnceLock::new(),
        }
    }

    fn payload(&'static self) -> &'static [u8] {
        self.data
            .get_or_init(|| generate_payload(self.spec.size).into_boxed_slice())
            .as_ref()
    }
}

static DATASET_STORAGE: [DatasetStorage; 3] = [
    DatasetStorage::new(DATASET_SPEC_LIST[0]),
    DatasetStorage::new(DATASET_SPEC_LIST[1]),
    DatasetStorage::new(DATASET_SPEC_LIST[2]),
];

macro_rules! register_digest_bench {
    ($fn_name:ident, $ty:ty, $name:literal) => {
        #[divan::bench(name = $name, args = DATASET_SPECS)]
        fn $fn_name(bencher: Bencher, dataset: DatasetSpec) {
            bench_digest::<$ty>(bencher, dataset);
        }
    };
}

register_digest_bench!(bench_crc32, Crc32, "crc32");
register_digest_bench!(bench_crc32c, Crc32c, "crc32c");

fn bench_digest<D>(bencher: Bencher, dataset: DatasetSpec)
where
    D: Digest + 'static,
{
    let payload = dataset_payload(dataset);
    bencher
        .counter(BytesCount::from(payload.len() as u64))
        .bench(|| {
            let mut digest = D::new();
            digest.update(payload);
            black_box(digest.finalize());
        });
}

fn dataset_payload(spec: DatasetSpec) -> &'static [u8] {
    DATASET_STORAGE
        .iter()
        .find(|storage| storage.spec == spec)
        .unwrap_or_else(|| panic!("unknown dataset: {}", spec.label))
        .payload()
}

fn generate_payload(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let mut state = 0x1234_5678u64;
    for _ in 0..size {
        state ^= state << 7;
        state ^= state >> 9;
        state ^= state << 8;
        data.push((state & 0xFF) as u8);
    }
    data
}
