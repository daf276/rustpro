// Writes the TCG Advanced pool as Pool::advanced computes it, one "code<TAB>ported" line per card, for tools/queue.py:
// "yes", "no", or "no script" for a card without a Lua script (Normal Monsters), which needs no port.
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use carddata::{CardDatabase, Pool, banlist_limits};
use oracle::Scripts;

fn main() -> Result<(), Box<dyn Error>> {
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let cards = CardDatabase::load(&vendor.join("BabelCDB"))?;
    let banlist = std::fs::read_to_string(vendor.join("LFLists/0TCG.lflist.conf"))?;
    let pool = Pool::advanced(&cards, banlist_limits(&banlist));
    let lua = Scripts::load(&vendor.join("CardScripts"))?;
    let out = &mut std::io::stdout().lock();
    for &code in pool.main.iter().chain(&pool.extra) {
        let ported = match (scripts::card(code), lua.contains(&format!("c{code}.lua"))) {
            (Some(_), _) => "yes",
            (None, true) => "no",
            (None, false) => "no script",
        };
        writeln!(out, "{code}\t{ported}")?;
    }
    Ok(())
}
