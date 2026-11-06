use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::LazyLock;

use carddata::{CardDatabase, Pool, banlist_limits};
use oracle::{Duel, Message, Outcome, Scripts, Status, parse, play_random_duel};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

const MAX_DECISIONS: u32 = 5_000;

struct Vendor {
    cards: CardDatabase,
    scripts: Scripts,
    pool: Pool,
}

static VENDOR: LazyLock<Result<Vendor, String>> = LazyLock::new(|| load().map_err(|error| error.to_string()));

fn load() -> Result<Vendor, Box<dyn Error>> {
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let cards = CardDatabase::load(&vendor.join("BabelCDB"))?;
    let limits = banlist_limits(&std::fs::read_to_string(vendor.join("LFLists/0TCG.lflist.conf"))?);
    let pool = Pool::advanced(&cards, limits);
    Ok(Vendor {
        scripts: Scripts::load(&vendor.join("CardScripts"))?,
        cards,
        pool,
    })
}

fn play(seed: u64, observe: impl FnMut(&[u8])) -> Result<Outcome, Box<dyn Error>> {
    let Vendor { cards, scripts, pool } = VENDOR.as_ref().map_err(String::as_str)?;
    Ok(play_random_duel(cards, scripts, |rng| pool.random_deck(rng), seed, MAX_DECISIONS, observe)?.outcome)
}

#[test]
fn should_build_legal_random_decks_without_forbidden_cards() {
    let Vendor { pool, .. } = VENDOR.as_ref().unwrap();
    assert!(pool.main.iter().chain(&pool.extra).all(|&code| pool.limit(code) > 0));
    for seed in 0..100 {
        let deck = pool.random_deck(&mut StdRng::seed_from_u64(seed));
        assert_eq!(deck.main.len(), 40);
        assert!(deck.extra.len() <= 15);
        assert!(deck.main.iter().all(|code| pool.main.contains(code)) && deck.extra.iter().all(|code| pool.extra.contains(code)));
        let mut copies: HashMap<u32, u8> = HashMap::new();
        deck.main
            .iter()
            .chain(&deck.extra)
            .for_each(|&code| *copies.entry(code).or_default() += 1);
        assert!(
            copies.iter().all(|(&code, &count)| count <= pool.limit(code)),
            "seed {seed}: {copies:?}"
        );
    }
}

#[test]
fn should_await_selection_without_lua_errors_and_retry_on_invalid_response() {
    let Vendor { cards, scripts, pool } = VENDOR.as_ref().unwrap();
    let mut rng = StdRng::seed_from_u64(42);
    let mut duel = Duel::new(cards, scripts, rng.random()).unwrap();
    for team in 0..2 {
        duel.add_deck(team, &pool.random_deck(&mut rng));
    }
    duel.start();
    let mut status = Status::Continue;
    while status == Status::Continue {
        status = duel.process().unwrap();
    }
    assert_eq!(status, Status::Awaiting);
    assert!(matches!(duel.messages().last().map(parse), Some(Ok(Message::Prompt(_)))));
    assert_eq!(duel.take_errors(), Vec::<String>::new());
    duel.set_response(&(-2i32).to_ne_bytes());
    assert_eq!(duel.process().unwrap(), Status::Awaiting);
    assert_eq!(
        duel.messages().map(|message| parse(message).unwrap()).collect::<Vec<_>>(),
        [Message::Retry]
    );
}

#[test]
fn should_produce_identical_message_streams_for_same_seed() {
    let stream = |seed| {
        let mut messages = Vec::new();
        let outcome = play(seed, |message| messages.push(message.to_vec())).unwrap();
        (outcome, messages)
    };
    for seed in 0..3 {
        assert_eq!(stream(seed), stream(seed));
    }
}

#[test]
fn should_answer_every_prompt_validly_in_random_duels() {
    let outcomes: Vec<Outcome> = (0..300).into_par_iter().map(|seed| play(seed, |_| {}).unwrap()).collect();
    let rejected: Vec<&Outcome> = outcomes
        .iter()
        .filter(|outcome| matches!(outcome, Outcome::Retry { .. } | Outcome::Unanswerable(_)))
        .collect();
    assert!(rejected.is_empty(), "{rejected:#?}");
}
