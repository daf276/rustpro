use std::error::Error;
use std::sync::Mutex;
use std::time::Instant;

use carddata::{CardDatabase, Deck, Decks, ELFNOTE_DECK, load_advanced, vendor};
use engine::{Action, DUEL_MODE_MR5, Duel, DuelError, DuelOptions, PlayerOptions, Status};
use fiber::Fiber;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use sim::{AutoAdvance, Batch, Options, indices};

#[cfg(not(feature = "system-allocator"))]
pub use mimalloc::MiMalloc as Allocator;
#[cfg(feature = "system-allocator")]
pub use std::alloc::System as Allocator;

pub type BenchResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub const MAX_ACTIONS: u64 = 100_000;

pub fn flags<const N: usize>(names: [&str; N], defaults: [u64; N]) -> BenchResult<[u64; N]> {
    let mut values = defaults;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("missing value")?.parse()?;
        let slot = names.iter().position(|&name| name == flag).and_then(|at| values.get_mut(at));
        *slot.ok_or_else(|| format!("unknown argument {flag}, expected {}", names.join(", ")))? = value;
    }
    Ok(values)
}

// Normal Monster decks from the Advanced pool, with the classic cards for `effects`, or the Elfnote deck for `elfnote`.
pub fn load_decks(effects: bool, elfnote: bool) -> BenchResult<(&'static CardDatabase, Decks)> {
    let (cards, mut pool) = load_advanced(&vendor())?;
    let cards: &'static CardDatabase = Box::leak(Box::new(cards));
    pool.retain_normal_monsters(cards);
    let effect_cards = if effects { scripts::CLASSIC_CARDS.to_vec() } else { Vec::new() };
    let deck = elfnote.then(|| Deck::parse(ELFNOTE_DECK, cards));
    Ok((cards, Decks { pool, effect_cards, deck }))
}

// A started duel of decks dealt from `decks`, and the RNG that then picks its actions.
pub fn setup(seed: u64, cards: &'static CardDatabase, decks: &Decks) -> Result<(Duel, StdRng), DuelError> {
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
        duel.add_deck(team, &decks.deal(&mut rng), cards)?;
    }
    duel.start();
    Ok((duel, rng))
}

// Plays uniformly random legal actions until the duel ends and returns how many it took; `at_prompt` sees every prompt.
pub fn play(duel: &mut Fiber, rng: &mut StdRng, mut at_prompt: impl FnMut(&Duel, &[Action], u64)) -> BenchResult<u64> {
    let mut legal = Vec::new();
    let mut status = duel.process();
    let mut actions = 0;
    while status == Status::Awaiting && actions < MAX_ACTIONS {
        duel.legal_actions_into(&mut legal);
        at_prompt(duel, &legal, actions);
        let &action = legal.choose(rng).ok_or("no legal action")?;
        status = duel.apply(action)?;
        actions += 1;
    }
    Ok(actions)
}

// Duels set up beforehand, stepped in batches of `size` slots at random; the actions, decisions among them and seconds.
pub fn run_batch(
    threads: usize,
    seeds: std::ops::Range<u64>,
    cards: &'static CardDatabase,
    decks: &Decks,
    size: usize,
    auto_advance: AutoAdvance,
) -> BenchResult<(u64, u64, f64)> {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let duels: Vec<Mutex<Option<Duel>>> = seeds
        .into_par_iter()
        .map(|seed| Ok(Mutex::new(Some(setup(seed, cards, decks)?.0))))
        .collect::<Result<_, DuelError>>()?;
    let generator = |slot: usize, episode: u64| Ok(duels.get(slot + size * episode as usize).and_then(|duel| duel.lock().ok()?.take()));
    let options = Options {
        auto_advance,
        max_steps: MAX_ACTIONS,
    };
    let mut rngs: Vec<StdRng> = (0..size as u64).map(StdRng::seed_from_u64).collect();
    let started = Instant::now();
    let outcomes = pool.install(|| {
        Batch::new(size, options, generator)?.run(|batch, actions| {
            for ((slot, action), rng) in actions.iter_mut().enumerate().zip(&mut rngs) {
                let legal = batch.legal(slot);
                if !legal.is_empty() {
                    *action = indices(legal)
                        .nth(rng.random_range(0..legal.len()))
                        .and_then(Result::ok)
                        .unwrap_or(0);
                }
            }
        })
    })?;
    let steps: u64 = outcomes.iter().map(|outcome| outcome.steps).sum();
    let skipped: u64 = outcomes.iter().map(|outcome| outcome.skipped).sum();
    Ok((steps + skipped, steps, started.elapsed().as_secs_f64()))
}
