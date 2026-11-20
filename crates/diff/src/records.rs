use std::error::Error;
use std::ops::Range;
use std::sync::OnceLock;

use engine::{Duel, DuelRecord};
use rayon::prelude::*;

use crate::{DiffError, Vendor, diff_duel, diff_scenario, sampled_cards};

type CheckResult = Result<(), Box<dyn Error + Send + Sync>>;

pub fn records(vendor: &'static Vendor, seeds: Range<u64>) -> Result<Vec<DuelRecord>, DiffError> {
    seeds.into_par_iter().map(|seed| Ok(diff_duel(vendor, seed)?.record)).collect()
}

// Scenario 0 of each card of [sampled_cards].
pub fn scenario_records(vendor: &'static Vendor) -> Result<Vec<DuelRecord>, DiffError> {
    sampled_cards()
        .into_par_iter()
        .map(|code| Ok(diff_scenario(vendor, code, 0)?.record))
        .collect()
}

// Elfnote mirror records and one scenario per ported card, built once per process for the tests that share them.
pub fn all_records() -> Result<&'static [(&'static Vendor, DuelRecord)], String> {
    static RECORDS: OnceLock<Result<Vec<(&'static Vendor, DuelRecord)>, String>> = OnceLock::new();
    let build = || -> Result<Vec<_>, DiffError> {
        let (elfnote, scenarios) = (Vendor::leak(Vendor::elfnote)?, Vendor::leak(Vendor::scenarios)?);
        let elfnote_records = records(elfnote, 0..40)?.into_iter().map(|record| (elfnote, record));
        let scenario_records = scenario_records(scenarios)?.into_iter().map(|record| (scenarios, record));
        Ok(elfnote_records.chain(scenario_records).collect())
    };
    let records = RECORDS.get_or_init(|| build().map_err(|error| error.to_string()));
    records.as_deref().map_err(String::clone)
}

// Calls `check` at every prompt of the record, before the recorded action.
pub fn each_state(vendor: &'static Vendor, record: &DuelRecord, mut check: impl FnMut(&Duel, usize) -> CheckResult) -> CheckResult {
    let mut duel = Duel::from_position(&record.position, &vendor.cards, &scripts::LIBRARY, record.seed)?;
    duel.process();
    for (index, &action) in record.actions.iter().enumerate() {
        check(&duel, index)?;
        duel.apply(action)?;
    }
    Ok(())
}

pub fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(0.0)
}
