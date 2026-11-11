mod cards;
mod pool;

use std::path::{Path, PathBuf};

pub use cards::*;
#[cfg(feature = "cdb")]
pub use pool::load_advanced;
pub use pool::{Deck, Decks, Pool, banlist_limits};

pub const ELFNOTE_DECK: &str = include_str!("../../diff/elfnote-ycs-montreal-2026.deck");

pub fn vendor() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor")
}
