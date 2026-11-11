// Generates the card registry from src/official: a module per c<code>.rs, CARD_SCRIPTS and the per-card const tables.
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
    // File-name order: with the sim profile's single codegen unit, code order measured 1-4% slower on the throughput bench.
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
    let tables: [(&str, &[&str], &str); 6] = [
        ("LISTED_NAMES", &["LISTED_NAMES", "FIT_MONSTER"], "&[u32]"),
        ("MATERIAL_SETCODE", &["MATERIAL_SETCODE"], "&[u16]"),
        ("MATERIAL", &["MATERIAL"], "&[u32]"),
        ("DARK_CALLING", &["DARK_CALLING"], "bool"),
        ("XYZ_NUMBER", &["XYZ_NUMBER"], "u32"),
        ("ASSAULT_MODE", &["ASSAULT_MODE"], "u32"),
    ];
    for (table, names, value) in tables {
        let rows: Vec<String> = cards
            .iter()
            .filter_map(|(code, text, _)| Some(format!("({code}, {})", field(*code, text, names)?)))
            .collect();
        writeln!(out, "pub static {table}: [(u32, {value}); {}] = [{}];", rows.len(), rows.join(", "))?;
    }
    std::fs::write(Path::new(&std::env::var("OUT_DIR")?).join("official.rs"), out)?;
    Ok(())
}
