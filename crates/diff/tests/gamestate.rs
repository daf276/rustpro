use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::time::{Duration, Instant};

use diff::{DiffError, Vendor, diff_duel, diff_position, median, records, scenario_records};
use engine::constants::{LOCATION_MZONE, PHASE_DRAW};
use engine::{Duel, DuelRecord, NewCardInfo, Position};
use rayon::prelude::*;
use scripts::LIBRARY;

type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

// Records are replayed from every STEP-th prefix.
const STEP: usize = 25;

fn prefix(record: &DuelRecord, actions: usize) -> DuelRecord {
    DuelRecord {
        actions: record.actions.iter().take(actions).copied().collect(),
        ..record.clone()
    }
}

// The duel replayed from every STEP-th prefix shows the record's position, rebuilds from it and plays on identically.
fn assert_replays(vendor: &'static Vendor, record: &DuelRecord) -> Result {
    let mut full = Duel::from_position(&record.position, &vendor.cards, &LIBRARY, record.seed)?;
    full.set_trace(true);
    full.process();
    let (mut trace, mut starts) = (Vec::new(), Vec::new());
    for (index, &action) in record.actions.iter().enumerate() {
        trace.extend(full.take_trace());
        if index % STEP == 0 {
            starts.push((index, trace.len(), full.position()));
        }
        full.apply(action)?;
    }
    trace.extend(full.take_trace());
    for (index, start, position) in starts {
        let mut duel = prefix(record, index).replay(&vendor.cards, &LIBRARY)?;
        assert_eq!(duel.position(), position, "seed {:?} at {index}", record.seed);
        let rebuilt = Duel::from_position(&position, &vendor.cards, &LIBRARY, record.seed)?;
        assert_eq!(rebuilt.position(), position, "seed {:?} at {index}", record.seed);
        duel.set_trace(true);
        record
            .actions
            .iter()
            .skip(index)
            .try_for_each(|&action| duel.apply(action).map(drop))?;
        let onward = trace.get(start..).unwrap_or_default();
        assert!(
            duel.take_trace() == onward,
            "seed {:?}: replayed at {index}, the messages differ",
            record.seed
        );
        assert_eq!(duel.result(), full.result());
    }
    Ok(())
}

#[test]
fn should_continue_identically_from_a_replayed_record_prefix() {
    let suites = [
        (Vendor::leak(Vendor::normal_monsters), 0..40),
        (Vendor::leak(Vendor::classic_effects), 0..40),
        (Vendor::leak(Vendor::elfnote), 0..80),
    ];
    for (vendor, seeds) in suites {
        let vendor = vendor.unwrap();
        let records = records(vendor, seeds).unwrap();
        records.par_iter().try_for_each(|record| assert_replays(vendor, record)).unwrap();
    }
    let vendor = Vendor::leak(Vendor::scenarios).unwrap();
    let records = scenario_records(vendor).unwrap();
    records.par_iter().try_for_each(|record| assert_replays(vendor, record)).unwrap();
}

// Whether a monster is under its non-owner's control, which a position cannot carry without the effect that took it.
fn is_taken(position: &Position) -> bool {
    let mut zones = HashSet::new();
    let mut monster = |info: &NewCardInfo| info.loc == u32::from(LOCATION_MZONE) && zones.insert((info.con, info.seq));
    position
        .cards
        .iter()
        .any(|card| monster(&card.info) && card.info.team != card.info.con)
}

// Positions of states a third and two thirds into each card's scenario, set to start the duel, built in both engines.
#[test]
fn should_match_the_oracle_from_positions_of_random_states() {
    let vendor = Vendor::leak(Vendor::scenarios).unwrap();
    let records = scenario_records(vendor).unwrap();
    let states: Vec<(u64, &DuelRecord, usize)> = (0..)
        .zip(
            records
                .iter()
                .flat_map(|record| [1, 2].map(|third| (record, record.actions.len() * third / 3))),
        )
        .map(|(seed, (record, actions))| (seed, record, actions))
        .collect();
    let failures: Vec<String> = states
        .par_iter()
        .map(|&(seed, record, actions)| -> Result {
            let position = Position {
                turn: 1,
                turn_player: 0,
                phase: PHASE_DRAW,
                ..prefix(record, actions).replay(&vendor.cards, &LIBRARY)?.position()
            };
            // Both engines agreed up to the cap: a random responder can wander for long (Gordian Slicer's select-all prompt).
            if is_taken(&position) {
                return Ok(());
            }
            match diff_position(vendor, seed, &position) {
                Ok(_) | Err(DiffError::DecisionCap { .. }) => Ok(()),
                Err(error) => Err(error.into()),
            }
        })
        .filter_map(|result| result.err().map(|error| error.to_string()))
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {}:\n{}",
        failures.len(),
        states.len(),
        failures.join("\n")
    );
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "a damaged record can be valid and reach an unported call site, which debug builds assert on"
)]
fn should_reject_damaged_bytes_without_panicking() {
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    let record = prefix(&diff_duel(vendor, 0).unwrap().record, 40);
    let bytes = record.to_bytes().unwrap();
    assert_eq!(DuelRecord::from_bytes(&bytes), Ok(record.clone()));
    assert!(Position::from_bytes(&bytes).is_err());
    let truncated = (0..bytes.len()).filter_map(|len| bytes.get(..len));
    assert!(truncated.into_iter().all(|bytes| DuelRecord::from_bytes(bytes).is_err()));
    // Each damaged byte gives an error or another record, which replays to some duel or an error.
    (0..bytes.len()).into_par_iter().for_each(|at| {
        let mut damaged = bytes.clone();
        if let Some(byte) = damaged.get_mut(at) {
            *byte ^= 0x5a;
        }
        if let Ok(record) = DuelRecord::from_bytes(&damaged) {
            drop(record.replay(&vendor.cards, &LIBRARY));
        }
    });
}

fn time<T>(run: impl Fn() -> T) -> (T, Duration) {
    let started = Instant::now();
    let value = run();
    (value, started.elapsed())
}

// Median sizes and times per state of Elfnote mirror matches, at every STEP-th action (single thread).
#[test]
#[ignore = "report, run with --release -- --ignored --nocapture --test-threads 1"]
fn report_gamestate_sizes_and_times() {
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    let mut rows: [Vec<f64>; 6] = Default::default();
    for record in records(vendor, 0..20).unwrap() {
        for index in (0..record.actions.len()).step_by(STEP) {
            let partial = prefix(&record, index);
            let (duel, replay) = time(|| partial.replay(&vendor.cards, &LIBRARY).unwrap());
            let (_, clone) = time(|| std::hint::black_box(duel.clone()));
            let (position, extract) = time(|| duel.position());
            let (_, rebuild) = time(|| Duel::from_position(&position, &vendor.cards, &LIBRARY, record.seed).unwrap());
            let sizes = [partial.to_bytes().unwrap().len(), position.to_bytes().unwrap().len()];
            let times = [replay, clone, extract, rebuild].map(|time| time.as_secs_f64() * 1e6);
            for (row, value) in rows.iter_mut().zip(sizes.map(|size| size as f64).into_iter().chain(times)) {
                row.push(value);
            }
        }
    }
    let states = rows.first().map_or(0, Vec::len);
    let [record, position, replay, clone, extract, rebuild] = rows.map(median);
    writeln!(
        std::io::stdout(),
        "{states} states: record {record:.0} B, replay {replay:.0} us; position {position:.0} B, extract {extract:.1} us, \
         rebuild {rebuild:.0} us; clone {clone:.1} us"
    )
    .unwrap();
}
