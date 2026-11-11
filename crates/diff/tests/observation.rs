use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::time::Instant;

use carddata::{Deck, TYPE_EXTRA, TYPE_MONSTER};
use diff::{
    DiffError, Vendor, diff_duel, diff_duel_probed, diff_scenario_probed, each_state, median, records, sampled_cards, scenario_records,
};
use engine::constants::*;
use engine::{Action, DeterminizeError, Duel, DuelRecord, PlacedCard, Position, Prompt, SeenCard, Status, Xoshiro256StarStar};
use rand::rngs::StdRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use scripts::LIBRARY;

type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

// States are checked at every STEP-th action of a record.
const STEP: usize = 25;
const MAX_ACTIONS: usize = 20_000;

// What `team` owns in the position: its decklist, as the player facing it may know it.
fn pool(vendor: &Vendor, position: &Position, team: u8) -> Deck {
    let is_extra = |code: u32| {
        let card_type = vendor.cards.get(code).map_or(0, |card| card.card_type);
        card_type & TYPE_MONSTER != 0 && card_type & TYPE_EXTRA != 0
    };
    let codes = position
        .cards
        .iter()
        .filter(|card| card.info.team == team)
        .map(|card| card.info.code);
    let (extra, main) = codes.partition(|&code| is_extra(code));
    Deck { main, extra }
}

// Random legal actions to the end; returns whether an effect reached an unported call site.
fn play_out(mut duel: Duel, rng: &mut StdRng) -> Result<bool> {
    let mut status = duel.process();
    for _ in 0..MAX_ACTIONS {
        if status == Status::End {
            break;
        }
        let legal = duel.legal_actions();
        // Uniform toggles walk an exact-count aux.SelectUnselectGroup for thousands of steps, so finish or mostly add cards.
        let selectable = duel.selectable_count();
        let picks: Vec<Action> = legal
            .iter()
            .copied()
            .filter(|&action| matches!(action, Action::Card(index) if usize::from(index) < selectable))
            .collect();
        let action = match duel.prompt() {
            Some(Prompt::UnselectCard { .. }) if legal.contains(&Action::Finish) => Action::Finish,
            Some(Prompt::UnselectCard { .. }) if !picks.is_empty() && rng.random_bool(0.9) => *picks.choose(rng).ok_or("no pick")?,
            _ => *legal.choose(rng).ok_or("a prompt without legal actions")?,
        };
        status = duel.apply(action)?;
    }
    Ok(duel.unported_site().is_some())
}

// At every STEP-th prompt, a duel determinized for either player shows that player the same and plays on without errors.
fn check_determinized(vendor: &'static Vendor, record: &DuelRecord) -> Result<[usize; 3]> {
    let seed = record.seed;
    let (mut rng, mut sampler) = (StdRng::seed_from_u64(seed[0]), Xoshiro256StarStar::new(seed));
    let mut counts = [0; 3];
    each_state(vendor, record, |duel, index| {
        for player in (0..2).filter(|_| index % STEP == 0) {
            let sample = match duel.determinize(player, &mut sampler, &pool(vendor, &record.position, 1 - player)) {
                Err(DeterminizeError::Rerun) => {
                    counts[2] += 1;
                    continue;
                }
                sample => sample?,
            };
            if sample.observe(player) != duel.observe(player) {
                return Err(format!("seed {seed:?} at {index}: player {player} sees a difference").into());
            }
            counts[1] += usize::from(play_out(sample, &mut rng)?);
            counts[0] += 1;
        }
        Ok(())
    })?;
    Ok(counts)
}

#[test]
fn should_determinize_into_duels_that_show_the_same_and_play_on() {
    let elfnote = Vendor::leak(Vendor::elfnote).unwrap();
    let scenarios = Vendor::leak(Vendor::scenarios).unwrap();
    let suites = [
        (elfnote, records(elfnote, 0..40).unwrap()),
        (scenarios, scenario_records(scenarios).unwrap()),
    ];
    for (vendor, records) in suites {
        let checked: Vec<[usize; 3]> = records
            .par_iter()
            .map(|record| check_determinized(vendor, record))
            .collect::<Result<_>>()
            .unwrap();
        let [samples, unported, failed] = checked.iter().fold([0; 3], |[a, b, c], [d, e, f]| [a + d, b + e, c + f]);
        writeln!(
            std::io::stdout(),
            "{samples} determinized states played out, {unported} reached unported call sites, {failed} not resampled"
        )
        .unwrap();
        assert!(
            unported * 100 <= samples && failed * 100 <= samples,
            "{unported} and {failed} of {samples}"
        );
    }
}

// The messages of a record's replay, observing and determinizing for both players at every prompt if `looking`.
fn replayed_trace(vendor: &'static Vendor, record: &DuelRecord, looking: bool) -> Result<Vec<u8>> {
    let mut duel = Duel::from_position(&record.position, &vendor.cards, &LIBRARY, record.seed)?;
    duel.set_trace(true);
    duel.process();
    let mut sampler = Xoshiro256StarStar::new(record.seed);
    for &action in &record.actions {
        for player in (0..2).filter(|_| looking) {
            duel.observe(player);
            duel.determinize(player, &mut sampler, &pool(vendor, &record.position, 1 - player))
                .ok();
        }
        duel.apply(action)?;
    }
    Ok(duel.take_trace())
}

#[test]
fn should_play_the_same_whether_observed_or_not() {
    let elfnote = Vendor::leak(Vendor::elfnote).unwrap();
    let scenarios = Vendor::leak(Vendor::scenarios).unwrap();
    for (vendor, records) in [
        (elfnote, records(elfnote, 0..40).unwrap()),
        (scenarios, scenario_records(scenarios).unwrap()),
    ] {
        records.par_iter().enumerate().for_each(|(index, record)| {
            let [observed, unobserved] = [true, false].map(|looking| replayed_trace(vendor, record, looking).unwrap());
            assert!(observed == unobserved, "record {index} plays differently when observed");
        });
    }
}

// The duel at player 0's first prompt, player 1 passing (or taking its first action) until then.
fn first_prompt_of_player_0(vendor: &'static Vendor, position: &Position, seed: [u64; 4]) -> Result<Duel> {
    let mut duel = Duel::from_position(position, &vendor.cards, &LIBRARY, seed)?;
    duel.process();
    while duel.prompt().is_some_and(|prompt| prompt.player() == 1) {
        let actions = duel.legal_actions();
        let pass = actions.iter().find(|&&action| action == Action::Chain(None));
        duel.apply(*pass.or(actions.first()).ok_or("a prompt without legal actions")?)?;
    }
    Ok(duel)
}

// Duels differing only in player 1's Deck order look the same to player 0 at its first prompt, player 1 passing until then.
#[test]
fn should_show_nothing_of_the_opponents_hidden_cards() {
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    for seed in 0..20 {
        let record = diff_duel(vendor, seed).unwrap().record;
        let mut position = record.position.clone();
        let mut deck: Vec<&mut PlacedCard> = position
            .cards
            .iter_mut()
            .filter(|card| card.info.team == 1 && card.info.loc == u32::from(LOCATION_DECK))
            .collect();
        let mut codes: Vec<u32> = deck.iter().map(|card| card.info.code).collect();
        codes.shuffle(&mut StdRng::seed_from_u64(seed));
        deck.iter_mut().zip(codes).for_each(|(card, code)| card.info.code = code);
        let [original, shuffled] =
            [&record.position, &position].map(|position| first_prompt_of_player_0(vendor, position, record.seed).unwrap());
        assert_eq!(original.observe(0), shuffled.observe(0), "seed {seed}");
        assert_ne!(original.observe(1), shuffled.observe(1), "seed {seed}");
    }
}

fn codes_in(position: &Position, team: u8, location: u8) -> impl Iterator<Item = u32> + '_ {
    let cards = position.cards.iter();
    cards
        .filter(move |card| card.info.team == team && card.info.loc == u32::from(location))
        .map(|card| card.info.code)
}

// The opponent's hand is drawn evenly from its decklist, its Extra Deck kept and player 0's own Deck reshuffled.
#[test]
fn should_sample_the_hidden_pool_evenly() {
    const SAMPLES: usize = 4000;
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    let record = diff_duel(vendor, 0).unwrap().record;
    let duel = first_prompt_of_player_0(vendor, &record.position, record.seed).unwrap();
    let pool = pool(vendor, &record.position, 1);
    let start = duel.position();
    let (mut hands, mut tops) = (HashMap::new(), HashMap::new());
    let mut sampler = Xoshiro256StarStar::new([1, 2, 3, 4]);
    for _ in 0..SAMPLES {
        let sample = duel.determinize(0, &mut sampler, &pool).unwrap().position();
        for code in codes_in(&sample, 1, LOCATION_HAND) {
            *hands.entry(code).or_insert(0.0) += 1.0;
        }
        let mut extra: Vec<u32> = codes_in(&sample, 1, LOCATION_EXTRA).collect();
        extra.sort_unstable();
        let mut expected: Vec<u32> = codes_in(&start, 1, LOCATION_EXTRA).collect();
        expected.sort_unstable();
        assert_eq!(extra, expected);
        *tops.entry(codes_in(&sample, 0, LOCATION_DECK).last().unwrap_or(0)).or_insert(0.0) += 1.0;
    }
    let hand = codes_in(&start, 1, LOCATION_HAND).count() as f64;
    let mut copies: HashMap<u32, f64> = HashMap::new();
    pool.main.iter().for_each(|&code| *copies.entry(code).or_insert(0.0) += 1.0);
    for (&code, &count) in &copies {
        let expected = SAMPLES as f64 * hand * count / pool.main.len() as f64;
        let seen = hands.get(&code).copied().unwrap_or(0.0);
        assert!(
            (seen - expected).abs() <= 5.0 * expected.sqrt(),
            "{code} in hand {seen} times, {expected:.0} expected"
        );
    }
    let own: Vec<u32> = codes_in(&start, 0, LOCATION_DECK).collect();
    for code in &own {
        let expected = SAMPLES as f64 * own.iter().filter(|&other| other == code).count() as f64 / own.len() as f64;
        let seen = tops.get(code).copied().unwrap_or(0.0);
        assert!(
            (seen - expected).abs() <= 5.0 * expected.sqrt(),
            "{code} on top {seen} times, {expected:.0} expected"
        );
    }
}

// Each queried card shows where the core has it, its code if QUERY_IS_PUBLIC or controlled outside the Deck, as EDOPro sends it.
fn probe_observations(oracle: &mut oracle::Duel<'_>, engine: &Duel) -> std::result::Result<(), String> {
    let observations = [engine.observe(0), engine.observe(1)];
    let locations = [
        LOCATION_DECK,
        LOCATION_HAND,
        LOCATION_MZONE,
        LOCATION_SZONE,
        LOCATION_GRAVE,
        LOCATION_REMOVED,
        LOCATION_EXTRA,
    ];
    let mut queried = 0;
    for controller in 0..2 {
        for location in locations {
            for (sequence, card) in oracle.query_location(controller, location.into()).into_iter().enumerate() {
                let Some(card) = card else { continue };
                queried += 1;
                for (player, observation) in (0..).zip(&observations) {
                    let visible = card.public || (controller == player && location != LOCATION_DECK);
                    let monster = visible && location == LOCATION_MZONE && card.position & POS_FACEUP != 0;
                    let expected = SeenCard {
                        controller,
                        location,
                        sequence: sequence as u8,
                        position: card.position as u8,
                        code: if visible { card.code } else { 0 },
                        attack: if monster { card.attack } else { 0 },
                        defense: if monster { card.defense } else { 0 },
                    };
                    let place =
                        |seen: &&SeenCard| (seen.controller, seen.location, seen.sequence) == (controller, location, sequence as u8);
                    let seen = observation.cards.iter().find(place);
                    if seen != Some(&expected) {
                        return Err(format!("player {player} sees {seen:?}, the core has {expected:?}"));
                    }
                }
            }
        }
    }
    let located = observations.first().map_or(0, |observation| {
        observation
            .cards
            .iter()
            .filter(|seen| seen.location & LOCATION_OVERLAY == 0)
            .count()
    });
    if located != queried {
        return Err(format!("{located} cards observed, {queried} queried"));
    }
    Ok(())
}

#[test]
fn should_show_what_the_core_marks_public() {
    let elfnote = Vendor::leak(Vendor::elfnote).unwrap();
    let duels = (0..20)
        .into_par_iter()
        .map(|seed| diff_duel_probed(elfnote, seed, &mut probe_observations).map(drop));
    duels.collect::<std::result::Result<(), DiffError>>().unwrap();
    let scenarios = Vendor::leak(Vendor::scenarios).unwrap();
    let cards = sampled_cards().into_par_iter();
    let scenarios = cards.map(|code| diff_scenario_probed(scenarios, code, 0, &mut probe_observations).map(drop));
    scenarios.collect::<std::result::Result<(), DiffError>>().unwrap();
}

// Median times per state of Elfnote mirror matches at every STEP-th action (single thread).
#[test]
#[ignore = "report, run with --release -- --ignored --nocapture --test-threads 1"]
fn report_observe_and_determinize_times() {
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    let mut rows: [Vec<f64>; 3] = Default::default();
    let mut sampler = Xoshiro256StarStar::new([1, 2, 3, 4]);
    let mut rng = StdRng::seed_from_u64(0);
    for record in records(vendor, 0..20).unwrap() {
        let pools = [1, 0].map(|team| pool(vendor, &record.position, team));
        each_state(vendor, &record, |duel, index| {
            if index % STEP == 0 {
                let player = rng.random_range(0..2);
                let pool = pools.get(usize::from(player)).ok_or("no pool")?;
                let started = Instant::now();
                std::hint::black_box(duel.clone());
                let clone = started.elapsed();
                let observation = std::hint::black_box(duel.observe(player));
                let observe = started.elapsed() - clone;
                std::hint::black_box(duel.determinize(player, &mut sampler, pool)?);
                let determinize = started.elapsed() - clone - observe;
                drop(observation);
                for (row, time) in rows.iter_mut().zip([clone, observe, determinize]) {
                    row.push(time.as_secs_f64() * 1e6);
                }
            }
            Ok(())
        })
        .unwrap();
    }
    let states = rows.first().map_or(0, Vec::len);
    let [clone, observe, determinize] = rows.map(median);
    writeln!(
        std::io::stdout(),
        "{states} states: clone {clone:.1} us, observe {observe:.1} us, determinize {determinize:.1} us"
    )
    .unwrap();
}
