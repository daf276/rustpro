// Writes the TCG Advanced pool for tools/queue.py, a "code<TAB>yes|no|no script" line per card.
use std::error::Error;
use std::io::Write;

use carddata::{load_advanced, vendor};
use oracle::Scripts;

fn main() -> Result<(), Box<dyn Error>> {
    let (_, pool) = load_advanced(&vendor())?;
    let lua = Scripts::load(&vendor().join("CardScripts"))?;
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
