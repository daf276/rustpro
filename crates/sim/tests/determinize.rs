mod common;

use std::cell::Cell;
use std::io::Write;

use carddata::{Deck, TYPE_EXTRA, TYPE_MONSTER};
use common::{Result, Tracked, check_truth, follow_swapped};
use diff::{Vendor, all_records};
use engine::{Choice, DeterminizeError, Duel, PlacedCard, Position, Prompt, Xoshiro256StarStar};
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

// Worlds are drawn at every STEP-th prompt and played on for PLAYED random actions, and swaps made at every STEP-th idle prompt.
const STEP: usize = 25;
const PLAYED: usize = 4;

// The other player's cards in the position, per player.
fn pools(vendor: &Vendor, position: &Position) -> [Deck; 2] {
    let is_extra = |code: u32| {
        let card_type = vendor.cards.get(code).map_or(0, |card| card.card_type);
        card_type & TYPE_MONSTER != 0 && card_type & TYPE_EXTRA != 0
    };
    [1, 0].map(|team| {
        let codes = position
            .cards
            .iter()
            .filter(|card| card.info.team == team)
            .map(|card| card.info.code);
        let (extra, main) = codes.partition(|&code| is_extra(code));
        Deck { main, extra }
    })
}

// A world for `player` that keeps what its tracker remembers, None if no sample reran to what the player sees.
fn world(tracked: &Tracked, player: u8, pool: &Deck, seed: u64) -> Result<Option<Duel>> {
    let memory = tracked.tracker(player).memory(&tracked.duel);
    let mut rng = Xoshiro256StarStar::new([seed, 1, 2, 3]);
    match tracked.duel.determinizer_with(player, pool, &memory)?.determinize(&mut rng) {
        Err(DeterminizeError::Rerun) => Ok(None),
        world => Ok(Some(world?)),
    }
}

// The cards a determinizer for `player` keeps as they are, in their places.
fn kept(duel: &Duel, player: u8, pool: &Deck) -> Result<Vec<Option<PlacedCard>>> {
    let cards = duel.position().cards.into_iter().zip(duel.determinizer(player, pool)?.resampled());
    Ok(cards.map(|(card, drawn)| (!drawn).then_some(card)).collect())
}

// The prompt and its choices as the player it waits for sees them.
fn pending(duel: &Duel) -> Option<(Prompt, Vec<Choice>)> {
    duel.prompt().map(|prompt| (prompt, duel.observe(prompt.player()).choices))
}

// The world's cards, with the cards of `start` no place of the duel holds (materials of an Xyz Monster on its way from the Extra
// Deck), which the world keeps as they are.
fn holdings(world: &Duel, duel: &Duel, start: &Position) -> Position {
    let mut placed: Vec<(u8, u32)> = duel.position().cards.iter().map(|card| (card.info.team, card.info.code)).collect();
    let unplaced = start.cards.iter().filter(|card| {
        let at = placed.iter().position(|&other| other == (card.info.team, card.info.code));
        at.map(|at| placed.swap_remove(at)).is_none()
    });
    let mut position = world.position();
    position.cards.extend(unplaced.cloned());
    position
}

// The world holds what the player remembers, shows it the same, waits at the same prompt and plays on; returns how many of the
// opponent's hidden cards it remembers, None if not resampled.
fn check_world(tracked: &Tracked, player: u8, pool: &Deck, start: &Position, rng: &mut StdRng) -> Result<Option<usize>> {
    let Some(mut world) = world(tracked, player, pool, rng.random())? else {
        return Ok(None);
    };
    let remembered = check_truth(&world, tracked.tracker(player), &holdings(&world, &tracked.duel, start))?;
    if world.observe(player) != tracked.duel.observe(player) || world.prompt() != tracked.duel.prompt() {
        return Err(format!("player {player}'s world shows it something else").into());
    }
    for _ in 0..PLAYED {
        let Some(&action) = world.legal_actions().choose(rng) else { break };
        world.apply(action)?;
    }
    Ok(Some(remembered))
}

#[test]
fn should_determinize_worlds_that_keep_what_the_player_remembers_and_play_on() {
    let records = all_records().unwrap();
    let checked = records
        .par_iter()
        .enumerate()
        .map(|(index, (vendor, record))| -> Result<[usize; 3]> {
            let mut rng = StdRng::seed_from_u64(index as u64);
            let pools = pools(vendor, &record.position);
            let mut tracked = Tracked::new(vendor, record)?;
            let [mut worlds, mut remembered, mut reruns] = [0; 3];
            for (at, &action) in record.actions.iter().enumerate() {
                for (player, pool) in (0..2).zip(&pools).filter(|_| at % STEP == 0) {
                    let checked = check_world(&tracked, player, pool, &record.position, &mut rng);
                    match checked.map_err(|error| format!("record {index}, action {at}: {error}"))? {
                        Some(known) => [worlds, remembered] = [worlds + 1, remembered + known],
                        None => reruns += 1,
                    }
                }
                tracked.apply(action)?;
            }
            Ok([worlds, remembered, reruns])
        });
    let checked: Vec<[usize; 3]> = checked.collect::<Result<_>>().unwrap();
    let [worlds, remembered, reruns] = checked.into_iter().fold([0; 3], |[a, b, c], [d, e, f]| [a + d, b + e, c + f]);
    writeln!(
        std::io::stdout(),
        "{worlds} worlds played on, {:.2} of the opponent's hidden cards remembered, {reruns} not resampled",
        remembered as f64 / worlds as f64
    )
    .unwrap();
    assert!(worlds > 1000 && reruns * 100 <= worlds, "{worlds} worlds, {reruns} not resampled");
}

// Equal worlds hold the same cards in the same places (positions) and show each player the same with the same choices. A hidden
// card something refers to by identity keeps its code and place, and the other player's prompt and choices stay as they are, so
// duels differing in either are skipped. State hashes can still differ: a drawn card takes its place's statuses (a hand card's
// STATUS_EFFECT_ENABLED, which the real card's history set), and the processor's sets (the operated cards) keep naming a card
// that went hidden, wherever it is drawn.
#[test]
fn should_determinize_the_same_world_from_duels_that_differ_only_in_what_the_player_never_saw() {
    let records = all_records().unwrap();
    let checked = records
        .par_iter()
        .enumerate()
        .map(|(index, (vendor, record))| -> Result<[usize; 3]> {
            let mut rng = StdRng::seed_from_u64(index as u64);
            let pools = pools(vendor, &record.position);
            let mut tracked = Tracked::new(vendor, record)?;
            let (mut compared, skipped, hashed) = (0, Cell::new(0), Cell::new(0));
            for (at, &action) in record.actions.iter().enumerate() {
                if at % STEP == 0 && matches!(tracked.duel.prompt(), Some(Prompt::IdleCommand { .. })) {
                    let rest = record.actions.get(at..).unwrap_or_default();
                    for (player, pool) in (0..2).zip(&pools) {
                        let same = |original: &Tracked, copy: &Tracked| -> Result {
                            if pending(&original.duel) != pending(&copy.duel)
                                || kept(&original.duel, player, pool)? != kept(&copy.duel, player, pool)?
                            {
                                skipped.set(skipped.get() + 1);
                                return Ok(());
                            }
                            let shown = |tracked: &Tracked| -> Result<_> {
                                let world = world(tracked, player, pool, at as u64)?;
                                Ok(world
                                    .map(|world| ((world.position(), [world.observe(0), world.observe(1)]), world.state_hash()))
                                    .unzip())
                            };
                            let [(seen, hash), (other_seen, other_hash)] = [shown(original)?, shown(copy)?];
                            hashed.set(hashed.get() + usize::from(hash == other_hash));
                            (seen == other_seen).then_some(()).ok_or("draws different worlds".into())
                        };
                        let followed = follow_swapped(&tracked, player, rest, &mut rng, same);
                        compared += followed.map_err(|error| format!("record {index}, action {at}: {error}"))?;
                    }
                }
                tracked.apply(action)?;
            }
            Ok([compared - skipped.get(), hashed.get(), skipped.get()])
        });
    let checked: Vec<[usize; 3]> = checked.collect::<Result<_>>().unwrap();
    let [compared, hashed, skipped] = checked.into_iter().fold([0; 3], |[a, b, c], [d, e, f]| [a + d, b + e, c + f]);
    writeln!(
        std::io::stdout(),
        "{compared} worlds compared after a swap of hidden cards ({hashed} with equal state hashes), {skipped} skipped"
    )
    .unwrap();
    assert!(compared > 1000, "{compared} worlds compared");
}
