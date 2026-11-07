use std::error::Error;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use carddata::{CardDatabase, Deck, Pool, TYPE_NORMAL, TYPE_PENDULUM, banlist_limits};
use engine::{Action, DUEL_MODE_MR5, Duel, DuelError, DuelOptions, PlayerOptions, Status};
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use sim::{AutoAdvance, Batch, Options, indices};

pub type BenchResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub const MAX_ACTIONS: u64 = 100_000;

// The pool and the ported cards a deck mixes in, if any, or a fixed deck both players play.
pub struct Decks {
    pub pool: Pool,
    pub effect_cards: Vec<u32>,
    pub deck: Option<Deck>,
}

impl Decks {
    // Random Normal Monster decks from the Advanced pool, with the classic cards mixed in for `effects`, or the Elfnote
    // deck for `elfnote`.
    pub fn load(effects: bool, elfnote: bool) -> BenchResult<(&'static CardDatabase, Self)> {
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
        let effect_cards = if effects { scripts::CLASSIC_CARDS.to_vec() } else { Vec::new() };
        let deck = elfnote.then(|| Deck::parse(include_str!("../../diff/elfnote-ycs-montreal-2026.deck"), cards));
        Ok((cards, Self { pool, effect_cards, deck }))
    }
}

// A started duel of random Normal Monster decks, with the ported cards mixed in if any, or of the fixed deck shuffled,
// and the RNG that then picks its actions.
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
        let deck = match &decks.deck {
            Some(deck) => deck.shuffled(&mut rng),
            None => decks.pool.random_deck_with(&decks.effect_cards, &mut rng),
        };
        duel.add_deck(team, &deck, cards)?;
    }
    duel.start();
    Ok((duel, rng))
}

// Plays uniformly random legal actions until the duel ends and returns how many it took; `at_prompt` sees every prompt
// with its legal actions.
pub fn play(duel: &mut Duel, rng: &mut StdRng, mut at_prompt: impl FnMut(&Duel, &[Action], u64)) -> BenchResult<u64> {
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

// The duels set up beforehand and stepped in lockstep batches of `size` slots under a uniformly random policy;
// returns the actions applied, policy decisions among them, and the seconds taken.
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
