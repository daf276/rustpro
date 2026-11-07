// Diffs the scenarios of the given cards against the oracle and writes per card whether all pass, the first failing
// scenario (divergence or unported site), each check's counts and the resolutions per effect; exits with 1 on any
// failure. cargo run --release -p diff --bin card -- <code>... [--scenarios K] [--threads N] [--context N]
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use diff::{SCENARIOS_FULL, Vendor, diff_cards, summarize};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let (mut cards, mut count, mut threads, mut context) = (Vec::new(), SCENARIOS_FULL, 0, None);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenarios" => count = args.next().ok_or("missing value")?.parse()?,
            "--threads" => threads = args.next().ok_or("missing value")?.parse()?,
            "--context" => context = Some(args.next().ok_or("missing value")?.parse()?),
            code => cards.push(code.trim_start_matches('c').parse::<u32>()?),
        }
    }
    if cards.is_empty() {
        return Err("usage: card <code>... [--scenarios K] [--threads N (0 = all cores)] [--context N (default 5)]".into());
    }
    rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let mut vendor = Vendor::scenarios(&vendor)?;
    vendor.context = context.unwrap_or(vendor.context);
    let vendor: &'static Vendor = Box::leak(Box::new(vendor));
    let (ported, missing): (Vec<u32>, Vec<u32>) = cards.iter().partition(|&&code| scripts::card(code).is_some());
    let out = &mut std::io::stdout().lock();
    let mut passed = true;
    for code in &missing {
        let note = match (vendor.cards.get(*code), vendor.scripts.contains(&format!("c{code}.lua"))) {
            (None, _) => "FAIL: not in the card database".to_owned(),
            (Some(_), true) => format!("FAIL: not ported, no crates/scripts/src/official/c{code}.rs"),
            (Some(_), false) => "pass: no script, nothing to port".to_owned(),
        };
        passed &= note.starts_with("pass");
        writeln!(out, "{code} {note}")?;
    }
    for (&card, results) in ported.iter().zip(diff_cards(vendor, &ported, count)) {
        let (ok, summary) = summarize(card, &results);
        passed &= ok;
        write!(out, "{summary}")?;
    }
    Ok(if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}
