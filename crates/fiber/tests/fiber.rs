use std::error::Error;

use carddata::{CardDatabase, Deck, ELFNOTE_DECK, load_advanced, vendor};
use engine::{Action, DUEL_MODE_MR5, Duel, DuelOptions, PlayerOptions, Status};
use fiber::Fiber;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Played = (Vec<u8>, Option<(u8, u8)>);

const DECISIONS: usize = 20_000;

fn started(seed: u64, cards: &'static CardDatabase, deck: &Deck) -> Result<Duel> {
    let player = PlayerOptions::default();
    let options = DuelOptions {
        seed: [seed, 2, 3, 4],
        flags: DUEL_MODE_MR5,
        team1: player,
        team2: player,
    };
    let mut duel = Duel::new(options)?;
    duel.load_library(&scripts::LIBRARY);
    for team in 0..2 {
        duel.add_deck(team, deck, cards)?;
    }
    duel.set_trace(true);
    duel.start();
    Ok(duel)
}

// A legal action picked by a hash of the decision number, so that a clone goes on with its original's choices.
fn pick(duel: &Duel, decision: usize) -> Result<Action> {
    let legal = duel.legal_actions();
    Ok(*legal
        .get((decision.wrapping_mul(0x9e37_79b9) >> 8) % legal.len().max(1))
        .ok_or("no legal action")?)
}

// Plays on from `decision`, calling `waiting` at each prompt of a waiting function; the trace and the result.
fn finish(fiber: &mut Fiber, mut decision: usize, mut waiting: impl FnMut(&mut Fiber, usize) -> Result<()>) -> Result<Played> {
    let mut status = fiber.process();
    while status == Status::Awaiting && decision < DECISIONS {
        if fiber.is_waiting() {
            waiting(fiber, decision)?;
        }
        let action = pick(fiber, decision)?;
        status = match decision % 7 {
            0 => std::thread::scope(|scope| scope.spawn(|| fiber.apply(action)).join()).map_err(|_| "apply panicked")??,
            _ => fiber.apply(action)?,
        };
        decision += 1;
    }
    Ok((fiber.take_trace(), fiber.result()))
}

#[test]
fn should_play_as_the_plain_duel_when_cloned_dropped_or_interrupted_while_a_function_waits() {
    let cards: &'static CardDatabase = Box::leak(Box::new(load_advanced(&vendor()).unwrap().0));
    let deck = Deck::parse(ELFNOTE_DECK, cards);
    for seed in 1..6 {
        let mut plain = started(seed, cards, &deck).unwrap();
        let (mut status, mut decision) = (plain.process(), 0);
        while status == Status::Awaiting && decision < DECISIONS {
            status = plain.apply(pick(&plain, decision).unwrap()).unwrap();
            decision += 1;
        }
        let expected = (plain.take_trace(), plain.result());
        let mut waits = 0;
        let played = finish(&mut Fiber::new(started(seed, cards, &deck).unwrap()), 0, |fiber, decision| {
            waits += 1;
            match waits % 16 {
                1 => assert_eq!(
                    finish(&mut fiber.clone(), decision, |_, _| Ok(()))?,
                    expected,
                    "cloned at {decision}"
                ),
                5 => {
                    let mut dropped = fiber.clone();
                    dropped.apply(pick(fiber, decision)?)?;
                }
                9 => {
                    fiber.duel_mut();
                }
                _ => {}
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(played, expected, "seed {seed}");
        assert!(waits > 16, "seed {seed}: {waits} prompts of a waiting function");
    }
}
