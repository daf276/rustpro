use std::error::Error;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

use diff::{Vendor, each_state, records, scenario_records};
use engine::{Action, BattleCommand, Duel, IdleCommand, Observation, Position, Prompt, Status};
use fiber::Fiber;
use rayon::prelude::*;

type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
type Seen = (Status, Vec<Action>, u16, Option<u64>, [Observation; 2], Option<Prompt>, Vec<Action>);
// What the duel shows after a pass, the whole duel as debug-printed (effect ids and the field id counter included), its messages,
// then its messages and result after playing on.
type Played = (Seen, String, Vec<u8>, Vec<u8>, Option<(u8, u8)>);

// Turns are passed from every STEP-th prompt of a record, then played on for ONWARD decisions.
const STEP: usize = 7;
const ONWARD: usize = 40;
// A hold is checked from every HOLD_STEP-th prompt, every TO_END-th of those passed to the end of the duel, at most TURNS turns.
const HOLD_STEP: usize = 21;
const TO_END: usize = 5;
const TURNS: usize = 200;
const PASSES: [Action; 5] = [
    Action::Chain(None),
    Action::IdleCommand(IdleCommand::ToEnd),
    Action::BattleCommand(BattleCommand::ToEnd),
    Action::YesNo(false),
    Action::Finish,
];

// Duel::pass_action's rule written out over the policy actions, applied one prompt at a time up to the next turn; also whether
// a function waited at one of them.
fn pass_by_steps(duel: &mut Fiber, actions: &mut Vec<Action>) -> Result<(Status, bool)> {
    let turn = duel.turn();
    let (mut status, mut waited) = (duel.process(), false);
    while status == Status::Awaiting && duel.turn() == turn {
        waited |= duel.is_waiting();
        let legal = duel.policy_actions();
        let kept: Vec<Action> = legal.iter().copied().filter(|&action| !duel.is_redundant(action)).collect();
        let options = if kept.is_empty() { &legal } else { &kept };
        let pass = PASSES.into_iter().find(|action| options.contains(action));
        let Some(action) = pass.or(options.first().copied()) else { break };
        status = duel.apply(action)?;
        actions.push(action);
    }
    Ok((status, waited))
}

// ONWARD decisions picked by their number.
fn played_on(duel: &mut Fiber, status: Status, actions: Vec<Action>) -> Result<Played> {
    let seen = (
        status,
        actions,
        duel.turn(),
        duel.state_hash(),
        [duel.observe(0), duel.observe(1)],
        duel.prompt(),
        duel.legal_actions(),
    );
    let printed = format!("{:?}", **duel);
    let passed = duel.take_trace();
    let (mut status, mut decision) = (status, 0usize);
    while status == Status::Awaiting && decision < ONWARD {
        let legal = duel.legal_actions();
        let action = legal
            .get(decision.wrapping_mul(0x9e37_79b9) % legal.len().max(1))
            .ok_or("no legal action")?;
        status = duel.apply(*action)?;
        decision += 1;
    }
    Ok((seen, printed, passed, duel.take_trace(), duel.result()))
}

// Fiber::pass_turn and Duel::pass_turn leave the duel as answering their pass actions one at a time does, and it plays on alike.
fn assert_passes_alike(duel: &Duel, index: usize, counts: &[AtomicUsize; 3]) -> Result {
    let copy = || {
        let mut copy = duel.clone();
        copy.set_trace(true);
        copy
    };
    let (mut stepped, mut actions) = (Fiber::new(copy()), Vec::new());
    let (status, waited) = pass_by_steps(&mut stepped, &mut actions)?;
    let expected = played_on(&mut stepped, status, actions)?;
    let mut passed = Fiber::new(copy());
    let mut actions = Vec::new();
    let status = passed.pass_turn(&mut actions);
    let fiber = played_on(&mut passed, status, actions)?;
    let (mut plain, mut actions) = (copy(), Vec::new());
    let status = plain.pass_turn(&mut actions);
    let plain = played_on(&mut Fiber::new(plain), status, actions)?;
    if fiber != expected || plain != expected {
        return Err(format!(
            "at prompt {index}: the fiber's pass differs {}, the plain duel's {}",
            fiber != expected,
            plain != expected
        )
        .into());
    }
    for (count, hit) in counts.iter().zip([true, duel.state_hash().is_none(), waited]) {
        count.fetch_add(usize::from(hit), Ordering::Relaxed);
    }
    Ok(())
}

#[test]
fn should_pass_the_turn_as_passing_one_prompt_at_a_time() {
    let elfnote = Vendor::leak(Vendor::elfnote).unwrap();
    let scenarios = Vendor::leak(Vendor::scenarios).unwrap();
    let counts = Default::default();
    for (vendor, records) in [
        (elfnote, records(elfnote, 0..20).unwrap()),
        (scenarios, scenario_records(scenarios).unwrap()),
    ] {
        records.par_iter().enumerate().for_each(|(record_index, record)| {
            each_state(vendor, record, |duel, index| match index % STEP {
                0 => assert_passes_alike(duel, index, &counts),
                _ => Ok(()),
            })
            .map_err(|error| format!("record {record_index}: {error}"))
            .unwrap();
        });
    }
    // Passes from a function waiting at the prompt, and passes answering a function's prompt in place.
    let [passes, from_waiting, through_waiting] = counts.map(AtomicUsize::into_inner);
    let counted = format!("{passes} passes, {from_waiting} from a waiting function, {through_waiting} answering one");
    writeln!(std::io::stdout(), "{counted}").unwrap();
    assert!(from_waiting * 20 > passes && through_waiting * 20 > passes, "{counted}");
}

// Passes up to `turns` times to the other player's first idle prompt after the turn, holding it first if `hold`, then opens
// it: the position and its play on; also whether a hold stopped the duel.
fn pass_to_idle(duel: &Duel, turns: usize, hold: bool) -> Result<(Position, Played, bool)> {
    let mut copy = duel.clone();
    copy.set_trace(true);
    let (mut duel, mut actions, mut held) = (Fiber::new(copy), Vec::new(), false);
    duel.process();
    for _ in 0..turns {
        if duel.result().is_some() {
            break;
        }
        let player = 1 - duel.turn_player();
        if hold {
            duel.hold_idle(player);
        }
        duel.pass_turn(&mut actions);
        while let Some(prompt) = duel.prompt()
            && prompt != (Prompt::IdleCommand { player })
            && let Some(action) = duel.pass_action()
        {
            duel.apply(action)?;
            actions.push(action);
        }
        held |= duel.prompt().is_none() && duel.result().is_none();
    }
    let status = duel.process();
    Ok((duel.position(), played_on(&mut duel, status, actions)?, held))
}

#[test]
fn should_open_a_held_idle_prompt_as_passing_up_to_it() {
    let elfnote = Vendor::leak(Vendor::elfnote).unwrap();
    let scenarios = Vendor::leak(Vendor::scenarios).unwrap();
    let [passes, holds] = [AtomicUsize::new(0), AtomicUsize::new(0)];
    for (vendor, records) in [
        (elfnote, records(elfnote, 0..20).unwrap()),
        (scenarios, scenario_records(scenarios).unwrap()),
    ] {
        records.par_iter().enumerate().for_each(|(record_index, record)| {
            each_state(vendor, record, |duel, index| {
                if index % HOLD_STEP != 0 {
                    return Ok(());
                }
                let turns = if index % (HOLD_STEP * TO_END) == 0 { TURNS } else { 1 };
                let (position, played, _) = pass_to_idle(duel, turns, false)?;
                let (held_position, held_played, held) = pass_to_idle(duel, turns, true)?;
                if held_position != position || held_played != played {
                    return Err(format!("at prompt {index}, passing {turns} turns: the held pass differs").into());
                }
                passes.fetch_add(1, Ordering::Relaxed);
                holds.fetch_add(usize::from(held), Ordering::Relaxed);
                Ok(())
            })
            .map_err(|error| format!("record {record_index}: {error}"))
            .unwrap();
        });
    }
    let [passes, holds] = [passes, holds].map(AtomicUsize::into_inner);
    writeln!(std::io::stdout(), "{holds} of {passes} passes held an idle prompt").unwrap();
    assert!(holds * 2 > passes, "{holds} of {passes} passes held an idle prompt");
}
