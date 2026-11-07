// Generates the card registry from the files in src/official: one module per c<code>.rs and the sorted CARD_SCRIPTS,
// with each card's optional SUPPORT and CHECKS consts, and LISTED_NAMES from the cards that declare it (or FIT_MONSTER).
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let official = Path::new(&std::env::var("CARGO_MANIFEST_DIR")?).join("src/official");
    writeln!(std::io::stdout(), "cargo::rerun-if-changed={}", official.display())?;
    let mut cards = Vec::new();
    for entry in std::fs::read_dir(&official)? {
        let path = entry?.path();
        let code = path
            .file_stem()
            .and_then(|stem| stem.to_str()?.strip_prefix('c')?.parse::<u32>().ok());
        if let Some(code) = code {
            cards.push((code, std::fs::read_to_string(&path)?, path));
        }
    }
    // Modules in file-name order, as they were hand-written: with the sim profile's single codegen unit their order sets
    // the code layout, and code order measured 1-4% slower on the Elfnote throughput bench.
    cards.sort_by(|a, b| a.2.cmp(&b.2));
    let mut out = String::new();
    for (code, _, path) in &cards {
        writeln!(out, "#[path = {:?}]\nmod c{code};", path.display().to_string())?;
    }
    cards.sort_by_key(|&(code, ..)| code);
    let field = |code: u32, text: &str, names: &[&str]| {
        let name = names.iter().find(|name| text.contains(&format!("pub const {name}:")))?;
        Some(format!("c{code}::{name}"))
    };
    writeln!(out, "pub static CARD_SCRIPTS: [CardScript; {}] = [", cards.len())?;
    for (code, text, _) in &cards {
        let support = field(*code, text, &["SUPPORT"]).unwrap_or("&[]".to_owned());
        let checks = field(*code, text, &["CHECKS"]).unwrap_or("&[Check::Resolves]".to_owned());
        writeln!(
            out,
            "    CardScript {{ code: {code}, initial_effect: c{code}::initial_effect, support: {support}, checks: {checks} }},"
        )?;
    }
    writeln!(out, "];")?;
    let listed: Vec<String> = cards
        .iter()
        .filter_map(|(code, text, _)| Some(format!("({code}, {})", field(*code, text, &["LISTED_NAMES", "FIT_MONSTER"])?)))
        .collect();
    writeln!(
        out,
        "pub static LISTED_NAMES: [(u32, &[u32]); {}] = [{}];",
        listed.len(),
        listed.join(", ")
    )?;
    let setcodes: Vec<String> = cards
        .iter()
        .filter_map(|(code, text, _)| Some(format!("({code}, {})", field(*code, text, &["MATERIAL_SETCODE"])?)))
        .collect();
    writeln!(
        out,
        "pub static MATERIAL_SETCODE: [(u32, &[u16]); {}] = [{}];",
        setcodes.len(),
        setcodes.join(", ")
    )?;
    let dark_calling: Vec<String> = cards
        .iter()
        .filter_map(|(code, text, _)| Some(format!("({code}, {})", field(*code, text, &["DARK_CALLING"])?)))
        .collect();
    writeln!(
        out,
        "pub static DARK_CALLING: [(u32, bool); {}] = [{}];",
        dark_calling.len(),
        dark_calling.join(", ")
    )?;
    let xyz_numbers: Vec<String> = cards
        .iter()
        .filter_map(|(code, text, _)| Some(format!("({code}, {})", field(*code, text, &["XYZ_NUMBER"])?)))
        .collect();
    writeln!(
        out,
        "pub static XYZ_NUMBER: [(u32, u32); {}] = [{}];",
        xyz_numbers.len(),
        xyz_numbers.join(", ")
    )?;
    std::fs::write(Path::new(&std::env::var("OUT_DIR")?).join("official.rs"), out)?;
    Ok(())
}
