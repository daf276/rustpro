use std::error::Error;
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use carddata::{CardDatabase, Deck, Pool, TYPE_NORMAL, TYPE_PENDULUM, banlist_limits};
use engine::{DUEL_MODE_MR5, Duel, DuelError, DuelOptions, PlayerOptions, Status};
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

const MAX_ACTIONS: u64 = 100_000;
const CLONE_DUELS: u64 = 500;
const CLONE_EVERY: u64 = 16;
const CLONES: u32 = 32;
const WIN_BY_LP: u8 = 1;

// The pool and the ported cards a deck mixes in, if any, or a fixed deck both players play.
struct Decks {
    pool: Pool,
    effect_cards: Vec<u32>,
    deck: Option<Deck>,
}

// A started duel of random Normal Monster decks, with the ported cards mixed in if any, or of the fixed deck shuffled,
// and the RNG that then picks its actions.
fn setup(seed: u64, cards: &'static CardDatabase, decks: &Decks) -> Result<(Duel, StdRng), DuelError> {
    let mut rng = StdRng::seed_from_u64(seed);
    let player = PlayerOptions::default();
    let mut duel = Duel::new(DuelOptions {
        seed: rng.random(),
        flags: DUEL_MODE_MR5,
        team1: player,
        team2: player,
    })?;
    if !decks.effect_cards.is_empty() || decks.deck.is_some() {
        duel.load_library(&scripts::LIBRARY);
    }
    for team in 0..2 {
        let deck = match &decks.deck {
            Some(deck) => deck.shuffled(&mut rng),
            None => decks.pool.random_deck_with(&decks.effect_cards, &mut rng),
        };
        duel.add_deck(team, &deck, cards)?;
    }
    duel.start();
    Ok((duel, rng))
}

// Plays uniformly random legal actions until the duel ends and returns how many it took; `at_prompt` sees every prompt.
fn play(duel: &mut Duel, rng: &mut StdRng, mut at_prompt: impl FnMut(&Duel, u64)) -> Result<u64, Box<dyn Error + Send + Sync>> {
    let mut legal = Vec::new();
    let mut status = duel.process();
    let mut actions = 0;
    while status == Status::Awaiting && actions < MAX_ACTIONS {
        at_prompt(duel, actions);
        duel.legal_actions_into(&mut legal);
        let &action = legal.choose(rng).ok_or("no legal action")?;
        status = duel.apply(action)?;
        actions += 1;
    }
    Ok(actions)
}

struct Run {
    threads: usize,
    seconds: f64,
    actions: u64,
    by_lp: usize,
    capped: usize,
    unported: usize,
}

fn run(
    threads: usize,
    seeds: std::ops::Range<u64>,
    cards: &'static CardDatabase,
    decks: &Decks,
) -> Result<Run, Box<dyn Error + Send + Sync>> {
    let threads = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let duels: Vec<(Duel, StdRng)> = seeds
        .into_par_iter()
        .map(|seed| setup(seed, cards, decks))
        .collect::<Result<_, _>>()?;
    let started = Instant::now();
    let played = threads.install(|| {
        duels
            .into_par_iter()
            .map(|(mut duel, mut rng)| Ok((play(&mut duel, &mut rng, |_, _| {})?, duel.result(), duel.unported_site().is_some())))
            .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()
    })?;
    Ok(Run {
        threads: threads.current_num_threads(),
        seconds: started.elapsed().as_secs_f64(),
        actions: played.iter().map(|(actions, ..)| actions).sum(),
        by_lp: played
            .iter()
            .filter(|(_, result, _)| result.is_some_and(|(_, reason)| reason == WIN_BY_LP))
            .count(),
        capped: played.iter().filter(|&&(actions, ..)| actions >= MAX_ACTIONS).count(),
        unported: played.iter().filter(|&&(.., unported)| unported).count(),
    })
}

// Mean time of a clone (and its drop) at every CLONE_EVERY-th prompt of the first CLONE_DUELS duels.
fn clone_time(
    seeds: std::ops::Range<u64>,
    cards: &'static CardDatabase,
    decks: &Decks,
) -> Result<(Duration, u32), Box<dyn Error + Send + Sync>> {
    let (mut total, mut clones) = (Duration::ZERO, 0);
    for seed in seeds {
        let (mut duel, mut rng) = setup(seed, cards, decks)?;
        play(&mut duel, &mut rng, |duel, action| {
            if action % CLONE_EVERY == 0 {
                let started = Instant::now();
                for _ in 0..CLONES {
                    black_box(black_box(duel).clone());
                }
                total += started.elapsed();
                clones += CLONES;
            }
        })?;
    }
    Ok((total / clones.max(1), clones))
}

// Engine throughput under uniformly random legal actions in random Normal Monster duels, setup excluded: one thread,
// then `--threads` (0 = all cores). `--effects 1` mixes the cards of phase 2a into the decks; `--elfnote 1` plays
// Elfnote mirror matches (main and Extra Deck) instead. "Unported" counts the duels whose result cannot be trusted, since an
// effect reached an unported call site.
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (mut duels, mut threads, mut seed, mut effects, mut elfnote) = (5000, 0, 0, 0, 0);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value: u64 = args.next().ok_or("missing value")?.parse()?;
        match flag.as_str() {
            "--duels" => duels = value,
            "--threads" => threads = usize::try_from(value)?,
            "--seed" => seed = value,
            "--effects" => effects = value,
            "--elfnote" => elfnote = value,
            _ => {
                let expected = "--duels, --threads (0 = all cores), --seed, --effects or --elfnote";
                return Err(format!("unknown argument {flag}, expected {expected}").into());
            }
        }
    }
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let cards: &'static CardDatabase = Box::leak(Box::new(CardDatabase::load(&vendor.join("BabelCDB"))?));
    let mut pool = Pool::advanced(
        cards,
        banlist_limits(&std::fs::read_to_string(vendor.join("LFLists/0TCG.lflist.conf"))?),
    );
    pool.retain(|code| {
        cards
            .get(code)
            .is_some_and(|card| card.card_type & (TYPE_NORMAL | TYPE_PENDULUM) == TYPE_NORMAL)
    });
    let effect_cards = if effects == 0 {
        Vec::new()
    } else {
        scripts::CLASSIC_CARDS.to_vec()
    };
    let deck = (elfnote != 0).then(|| Deck::parse(include_str!("../../../diff/elfnote-ycs-montreal-2026.deck"), cards));
    let decks = Decks { pool, effect_cards, deck };
    let seeds = seed..seed + duels;

    let out = &mut std::io::stdout().lock();
    writeln!(out, "size_of::<Duel>() = {} B", size_of::<Duel>())?;
    let (clone, clones) = clone_time(seed..seed + duels.min(CLONE_DUELS), cards, &decks)?;
    writeln!(out, "clone: {} ns mean over {clones} clones", clone.as_nanos())?;
    let single = run(1, seeds.clone(), cards, &decks)?;
    let parallel = run(threads, seeds.clone(), cards, &decks)?;
    writeln!(
        out,
        "{duels} duels (seeds {seeds:?}), {} actions ({:.0} per duel), {} won by LP, {} capped at {MAX_ACTIONS} actions, {} unported",
        single.actions,
        single.actions as f64 / duels as f64,
        single.by_lp,
        single.capped,
        single.unported
    )?;
    for run in [&single, &parallel] {
        writeln!(
            out,
            "{:>2} threads: {:>10.0} actions/s, {:>8.0} duels/s, {:.2}x one thread",
            run.threads,
            run.actions as f64 / run.seconds,
            duels as f64 / run.seconds,
            single.seconds / run.seconds
        )?;
    }
    Ok(())
}
