use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

// The code the oracle's stream and the responder's choices depend on besides the vendored checkouts.
const SOURCES: [&str; 15] = [
    include_str!("lib.rs"),
    include_str!("scenario.rs"),
    include_str!("../../oracle/src/cards.rs"),
    include_str!("../../oracle/src/duel.rs"),
    include_str!("../../oracle/src/fuzz.rs"),
    include_str!("../../oracle/src/lib.rs"),
    include_str!("../../oracle/src/message.rs"),
    include_str!("../../oracle/src/random.rs"),
    include_str!("../../oracle/src/scripts.rs"),
    include_str!("../../ocgcore-sys/build.rs"),
    include_str!("../../ocgcore-sys/src/lib.rs"),
    include_str!("../../carddata/src/cards.rs"),
    include_str!("../../carddata/src/lib.rs"),
    include_str!("../../carddata/src/pool.rs"),
    include_str!("../../../Cargo.lock"),
];

// 128 bits from std: two SipHashes, salted apart.
pub(crate) struct Digest([DefaultHasher; 2]);

impl Digest {
    pub(crate) fn new() -> Self {
        Self([0, 1].map(|salt| {
            let mut hasher = DefaultHasher::new();
            hasher.write_u8(salt);
            hasher
        }))
    }

    pub(crate) fn value(&self) -> u128 {
        let [high, low] = self.0.each_ref().map(|hasher| hasher.finish());
        u128::from(high) << 64 | u128::from(low)
    }
}

impl Hasher for Digest {
    fn write(&mut self, bytes: &[u8]) {
        self.0.iter_mut().for_each(|hasher| hasher.write(bytes));
    }

    fn finish(&self) -> u64 {
        self.value() as u64
    }
}

fn digest(value: &impl Hash) -> u128 {
    let mut digest = Digest::new();
    value.hash(&mut digest);
    digest.value()
}

pub(crate) struct Entry(PathBuf);

impl Entry {
    pub(crate) fn read(&self) -> Option<u128> {
        Some(u128::from_le_bytes(std::fs::read(&self.0).ok()?.try_into().ok()?))
    }

    pub(crate) fn write(&self, hash: u128) {
        let temporary = self
            .0
            .with_extension(format!("{}-{:?}", std::process::id(), std::thread::current().id()));
        if std::fs::write(&temporary, hash.to_le_bytes())
            .and_then(|()| std::fs::rename(&temporary, &self.0))
            .is_err()
        {
            let _ = std::fs::remove_file(&temporary);
        }
    }
}

// None with ORACLE_CACHE=off or when the vendored checkouts' HEADs cannot be read.
pub(crate) fn entry(key: &impl Hash) -> Option<Entry> {
    static CACHE: OnceLock<Option<(PathBuf, u128)>> = OnceLock::new();
    let (directory, version) = CACHE.get_or_init(open).as_ref()?;
    Some(Entry(directory.join(format!("{:032x}", digest(&(version, key))))))
}

fn open() -> Option<(PathBuf, u128)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = match std::env::var_os("ORACLE_CACHE") {
        Some(value) if value == "off" => return None,
        Some(value) => PathBuf::from(value),
        None => root.join("target/oracle-cache"),
    };
    let heads: Option<Vec<Vec<u8>>> = ["ygopro-core", "CardScripts", "BabelCDB"]
        .iter()
        .map(|name| {
            let checkout = root.join("vendor").join(name);
            let output = Command::new("git")
                .arg("-C")
                .arg(checkout)
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()?;
            output.status.success().then_some(output.stdout)
        })
        .collect();
    let version = digest(&(SOURCES, heads?));
    std::fs::create_dir_all(&directory).ok()?;
    Some((directory, version))
}
