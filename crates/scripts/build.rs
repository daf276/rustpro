// Generates the card registry from src/official: a module per c<code>.rs, CARD_SCRIPTS and the per-card const tables.
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

// Identifiers in a card's own source that make its zones matter: sequences, columns, linked groups, a kind of Monster Zone,
// zones to summon to or disable, a return to the zone left. Counts of free zones do not, nor do Link arrows and the shared
// procedures (Pendulum, Link), which come with the card's type.
const ZONE_WORDS: [&str; 19] = [
    "seq",
    "column",
    "linked",
    "marker",
    "adjacent",
    "pointed",
    "zpt",
    "mmz",
    "emz",
    "zone",
    "MMZ",
    "EMZ",
    "COLUMN",
    "LINKED",
    "MARKER",
    "DISABLE_FIELD",
    "return_to_field",
    "field_return",
    "remove_until",
];
const COUNT_ONLY: [&str; 4] = [
    "get_mzone_count",
    "get_mzone_count_full",
    "get_usable_mzone_count",
    "check_pendulum_zones",
];
// Identifiers that check or change battle positions, besides those ending in `_pos`.
const POSITION_WORDS: [&str; 4] = ["position", "POSITION", "POSCHANGE", "CHANGE_POS"];
// The consts the registry reads from a card file, found only as `pub const NAME:` at a line's start, and their types.
const METADATA: [(&str, &str); 10] = [
    ("SUPPORT", "&[(u32,u16)]"),
    ("CHECKS", "&[Check]"),
    ("LISTED_NAMES", "&[u32]"),
    ("FIT_MONSTER", "&[u32]"),
    ("MATERIAL_SETCODE", "&[u16]"),
    ("MATERIAL", "&[u32]"),
    ("DARK_CALLING", "bool"),
    ("XYZ_NUMBER", "u32"),
    ("ASSAULT_MODE", "u32"),
    ("PENDULUM_LEVEL", "u32"),
];

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
            let text = std::fs::read_to_string(&path)?;
            let code_only: String = text
                .lines()
                .map(|line| line.split("//").next().unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            check_metadata(&path, &code_only)?;
            cards.push((code, code_only, path));
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
        let name = names
            .iter()
            .find(|name| text.lines().any(|line| line.starts_with(&format!("pub const {name}:"))))?;
        Some(format!("c{code}::{name}"))
    };
    let zones = sensitive(&cards, |word| {
        !COUNT_ONLY.contains(&word) && ZONE_WORDS.iter().any(|zone| word.contains(zone))
    });
    let positions = sensitive(&cards, |word| {
        word.ends_with("_pos") || POSITION_WORDS.iter().any(|position| word.contains(position))
    });
    writeln!(out, "pub static CARD_SCRIPTS: [CardScript; {}] = [", cards.len())?;
    for ((code, text, _), (uses_zones, uses_positions)) in cards.iter().zip(zones.into_iter().zip(positions)) {
        let support = field(*code, text, &["SUPPORT"]).unwrap_or("&[]".to_owned());
        let checks = field(*code, text, &["CHECKS"]).unwrap_or("&[Check::Resolves]".to_owned());
        writeln!(
            out,
            "    CardScript {{ code: {code}, initial_effect: c{code}::initial_effect, support: {support}, checks: {checks}, \
             uses_zones: {uses_zones}, uses_positions: {uses_positions} }},"
        )?;
    }
    writeln!(out, "];")?;
    let tables: [(&str, &[&str], &str); 8] = [
        ("LISTED_NAMES", &["LISTED_NAMES", "FIT_MONSTER"], "&[u32]"),
        ("COUNTER_PLACE_LIST", &["COUNTER_PLACE_LIST"], "&[u16]"),
        ("MATERIAL_SETCODE", &["MATERIAL_SETCODE"], "&[u16]"),
        ("MATERIAL", &["MATERIAL"], "&[u32]"),
        ("DARK_CALLING", &["DARK_CALLING"], "bool"),
        ("XYZ_NUMBER", &["XYZ_NUMBER"], "u32"),
        ("ASSAULT_MODE", &["ASSAULT_MODE"], "u32"),
        ("PENDULUM_LEVEL", &["PENDULUM_LEVEL"], "u32"),
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

// A metadata const declared in a form `field` misses, or a top-level const of a metadata type named a typo away from it.
fn check_metadata(path: &Path, text: &str) -> Result<(), String> {
    for (number, line) in text.lines().enumerate() {
        let Some((before, rest)) = line.split_once("const ") else {
            continue;
        };
        let name: String = rest.chars().take_while(|&c| c.is_alphanumeric() || c == '_').collect();
        let declared = METADATA.iter().any(|&(known, _)| known == name);
        let declaration = rest.strip_prefix(name.as_str()).and_then(|rest| rest.split_once('='));
        let kind: String = declaration
            .map_or("", |(kind, _)| kind)
            .chars()
            .filter(|&c| c != ':' && !c.is_whitespace())
            .collect();
        let misspelled = !declared
            && !before.starts_with(char::is_whitespace)
            && METADATA
                .iter()
                .any(|&(known, known_kind)| known_kind == kind && edit_distance(known, &name) <= 2);
        if (declared && !line.starts_with(&format!("pub const {name}:"))) || misspelled {
            return Err(format!(
                "{}:{}: `{name}` is not read as card metadata; declare it as `pub const NAME:` at the line's start with NAME one of {:?}",
                path.display(),
                number + 1,
                METADATA.map(|(known, _)| known)
            ));
        }
    }
    Ok(())
}

fn edit_distance(a: &str, b: &str) -> usize {
    let first: Vec<usize> = (0..=b.len()).collect();
    let last = a.bytes().enumerate().fold(first, |row, (index, from)| {
        let mut next = vec![index + 1];
        for ((to, &diagonal), &above) in b.bytes().zip(&row).zip(row.iter().skip(1)) {
            let left = next.last().copied().unwrap_or_default();
            next.push((diagonal + usize::from(from != to)).min(above + 1).min(left + 1));
        }
        next
    });
    last.last().copied().unwrap_or_default()
}

// Per card (sorted by code), whether an identifier in its source, or in the source of a card it uses (`super::c<code>`), matches.
fn sensitive(cards: &[(u32, String, PathBuf)], matches: impl Fn(&str) -> bool) -> Vec<bool> {
    let mentions = |text: &str| text.split(|c: char| !c.is_alphanumeric() && c != '_').any(&matches);
    let uses: Vec<Vec<usize>> = (cards.iter())
        .map(|(_, text, _)| {
            let codes = text.split("::c").skip(1).filter_map(|rest| {
                let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                digits.parse::<u32>().ok()
            });
            codes
                .filter_map(|code| cards.binary_search_by_key(&code, |&(code, ..)| code).ok())
                .collect()
        })
        .collect();
    let mut flags: Vec<bool> = cards.iter().map(|(_, text, _)| mentions(text)).collect();
    loop {
        let used = |other: &usize| flags.get(*other).copied().unwrap_or(false);
        let next: Vec<bool> = flags
            .iter()
            .zip(&uses)
            .map(|(&flag, others)| flag || others.iter().any(used))
            .collect();
        if next == flags {
            return flags;
        }
        flags = next;
    }
}
