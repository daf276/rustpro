// Diffs the given cards' scenarios, writing per card its checks and first failure; --trace K writes scenario K's messages.
// cargo run --release -p diff --bin card -- <code>... [--scenarios K] [--threads N] [--context N] [--trace K]
use std::collections::BTreeSet;
use std::error::Error;
use std::io::Write;
use std::process::ExitCode;

use diff::{SCENARIOS_FULL, Vendor, describe, diff_cards, diff_scenario, scenario, summarize};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let (mut cards, mut count, mut threads, mut context, mut trace) = (Vec::new(), SCENARIOS_FULL, 0, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenarios" => count = args.next().ok_or("missing value")?.parse()?,
            "--threads" => threads = args.next().ok_or("missing value")?.parse()?,
            "--context" => context = Some(args.next().ok_or("missing value")?.parse()?),
            "--trace" => trace = Some(args.next().ok_or("missing value")?.parse::<u32>()?),
            code => cards.push(code.trim_start_matches('c').parse::<u32>()?),
        }
    }
    if cards.is_empty() {
        return Err("usage: card <code>... [--scenarios K] [--threads N (0 = all cores)] [--context N (default 5)] [--trace K]".into());
    }
    rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
    let mut vendor = Vendor::scenarios(&carddata::vendor())?;
    vendor.context = context.unwrap_or(vendor.context);
    vendor.keep_stream = trace.is_some();
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
        if let Some(index) = trace {
            let placed = scenario(vendor, card, index).map(|scenario| scenario.cards).unwrap_or_default();
            writeln!(out, "  scenario {index} places (code, player, location, sequence, position):")?;
            for info in placed.iter().map(|card| card.info) {
                writeln!(out, "    {} {} {:#x} {} {:#x}", info.code, info.con, info.loc, info.seq, info.pos)?;
            }
            match diff_scenario(vendor, card, index) {
                Ok(report) => {
                    let codes: BTreeSet<u32> = report.record.position.cards.iter().map(|card| card.info.code).collect();
                    for message in &report.stream {
                        let named = message
                            .windows(4)
                            .filter_map(|bytes| Some(u32::from_le_bytes(bytes.try_into().ok()?)));
                        let named: Vec<String> = named.filter(|code| codes.contains(code)).map(|code| code.to_string()).collect();
                        writeln!(out, "    {} {}", describe(message), named.join(" "))?;
                    }
                }
                Err(error) => writeln!(out, "    {}", error.to_string().trim_end())?,
            }
        }
    }
    Ok(if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}
