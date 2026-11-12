// The search API on Elfnote mirror matches: per prompt kind, how often state_hash is Some at a decision and what it costs; the
// cards behind the legal actions by action_card against observe_into; with `--check 1`, that decisions of one duel hashing
// alike, or alike after two copies of a card swap places, look and play alike.
use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

use bench::{Allocator, BenchResult, flags, load_decks, play, setup};
use carddata::{CardDatabase, Decks};
use engine::constants::{LOCATION_EXTRA, LOCATION_GRAVE, LOCATION_HAND, LOCATION_REMOVED};
use engine::{Action, CardId, Choice, Duel, ObservationSink, Prompt, SeenCard, SeenLink, Status};
use fiber::Fiber;
use scripts::prelude::LOCATION_ALL;

#[global_allocator]
static GLOBAL: Allocator = Allocator;

const REPEATS: u32 = 8;
// Decisions played on from two states that hash alike, picking by the same rule in both.
const FUTURE: usize = 200;

#[derive(Default)]
struct Kind {
    decisions: u64,
    hashed: u64,
    nanos: Vec<u32>,
}

#[derive(Default)]
struct Measured {
    kinds: BTreeMap<String, Kind>,
    action_card: Vec<u32>,
    observe: Vec<u32>,
    repeats: u64,
    swaps_merged: u64,
    swaps_split: u64,
    mismatches: Vec<String>,
}

struct Choices(Vec<Choice>);

impl ObservationSink for Choices {
    fn card(&mut self, _: SeenCard) {}

    fn link(&mut self, _: &SeenLink) {}

    fn target(&mut self, _: SeenCard) {}

    fn choice(&mut self, choice: Choice) {
        self.0.push(choice);
    }
}

fn nanos<T>(mut call: impl FnMut() -> T) -> u32 {
    let started = Instant::now();
    for _ in 0..REPEATS {
        black_box(call());
    }
    u32::try_from(started.elapsed().as_nanos() / u128::from(REPEATS)).unwrap_or(u32::MAX)
}

// What both players see and the prompt's choices, without the order of the hand, the Extra Deck, chain targets and choices.
fn looks(duel: &Duel) -> String {
    let unordered = |card: &mut SeenCard| {
        if card.location & (LOCATION_HAND | LOCATION_EXTRA) != 0 {
            card.sequence = 0;
        }
    };
    let place = |card: &SeenCard| (card.controller, card.location, card.code, card.sequence, card.position);
    let views = (0..2).map(|player| {
        let mut observation = duel.observe(player);
        observation.cards.iter_mut().for_each(unordered);
        observation.cards.sort_by_key(place);
        for link in &mut observation.chain {
            link.targets.iter_mut().for_each(unordered);
            link.targets.sort_by_key(place);
        }
        observation
            .choices
            .iter_mut()
            .filter_map(|choice| choice.card.as_mut())
            .for_each(unordered);
        let mut choices: Vec<_> = observation
            .choices
            .iter()
            .map(|choice| (choice.card.map(|card| place(&card)), choice.description))
            .collect();
        choices.sort_unstable();
        observation.choices.clear();
        format!("{observation:?} {choices:?}")
    });
    views.collect::<Vec<_>>().join("\n")
}

// An action by what it does, not its index, which differs between states that order their options apart.
fn content(duel: &Duel, action: Action) -> String {
    let kind: String = format!("{action:?}").chars().filter(|c| !c.is_ascii_digit()).collect();
    match duel.action_card(action) {
        (Some(card), description) => {
            let location = card.get_location(duel);
            let sequence = if location & u32::from(LOCATION_HAND | LOCATION_EXTRA) == 0 {
                card.get_sequence(duel)
            } else {
                0
            };
            let place = (card.get_controler(duel), location, sequence, card.get_position(duel));
            format!("{kind} {} {place:?} {description}", card.get_original_code(duel))
        }
        (None, description) => format!("{action:?} {description}"),
    }
}

// How a duel looks over FUTURE decisions, each picked by the decision number among the actions in order of [content].
fn future(duel: &Duel) -> BenchResult<Vec<String>> {
    let mut duel = duel.clone();
    let mut looked = Vec::with_capacity(FUTURE);
    for decision in 0..FUTURE {
        let mut legal: Vec<(String, Action)> = duel
            .legal_actions()
            .into_iter()
            .map(|action| (content(&duel, action), action))
            .collect();
        legal.sort_by(|(first, _), (second, _)| first.cmp(second));
        let Some(&(_, action)) = legal.get(decision.wrapping_mul(0x9e37_79b9) % legal.len().max(1)) else {
            break;
        };
        if duel.apply(action)? == Status::End {
            break;
        }
        looked.push(looks(&duel));
    }
    Ok(looked)
}

// Why two duels hashing alike are not alike: they look apart, or apart after the same decisions.
fn consistent(first: &Duel, first_looks: &str, second: &Duel, second_looks: &str) -> BenchResult<Option<String>> {
    let apart = |first: &str, second: &str| {
        let at = first.bytes().zip(second.bytes()).take_while(|(a, b)| a == b).count();
        let around = |text: &str| {
            text.get(at.saturating_sub(150)..(at + 150).min(text.len()))
                .unwrap_or_default()
                .to_owned()
        };
        format!("{}\n{}", around(first), around(second))
    };
    if first_looks != second_looks {
        return Ok(Some(format!("they look apart:\n{}", apart(first_looks, second_looks))));
    }
    let (first, second) = (future(first)?, future(second)?);
    let decision = first.iter().zip(&second).position(|(first, second)| first != second);
    Ok(decision.map(|decision| {
        let (first, second) = (
            first.get(decision).map_or("", String::as_str),
            second.get(decision).map_or("", String::as_str),
        );
        format!("they play apart at decision {decision}:\n{}", apart(first, second))
    }))
}

// Two copies of a card in different locations, or both in the GY or banished, where their order and turn may differ.
fn copies(duel: &Duel) -> Option<(CardId, CardId)> {
    let cards: Vec<CardId> = duel.get_field_group(0, LOCATION_ALL, LOCATION_ALL).iter().collect();
    let apart = |card: CardId, other: CardId| {
        let (location, other_location) = (card.get_location(duel), other.get_location(duel));
        location != other_location || location & u32::from(LOCATION_GRAVE | LOCATION_REMOVED) != 0
    };
    let pairs = cards
        .iter()
        .enumerate()
        .flat_map(|(at, &card)| cards.iter().skip(at + 1).map(move |&other| (card, other)));
    pairs
        .filter(|&(card, other)| card.get_original_code(duel) == other.get_original_code(duel))
        .find(|&(card, other)| apart(card, other))
}

fn kind(prompt: Option<Prompt>) -> String {
    let name = format!("{prompt:?}");
    name.trim_start_matches("Some(")
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect()
}

fn measure(seed: u64, cards: &'static CardDatabase, decks: &Decks, check: bool, measured: &mut Measured) -> BenchResult<()> {
    let (duel, mut rng) = setup(seed, cards, decks)?;
    let mut duel = Fiber::new(duel);
    let mut seen: HashMap<u64, (Duel, String)> = HashMap::new();
    let mut choices = Choices(Vec::new());
    let mut failure = None;
    play(&mut duel, &mut rng, |duel, legal, _| {
        if legal.len() < 2 || failure.is_some() {
            return;
        }
        let entry = measured.kinds.entry(kind(duel.prompt())).or_default();
        entry.decisions += 1;
        let hash = duel.state_hash();
        if hash.is_some() {
            entry.hashed += 1;
            entry.nanos.push(nanos(|| duel.state_hash()));
        }
        measured
            .action_card
            .push(nanos(|| legal.iter().map(|&action| duel.action_card(action)).for_each(drop)));
        let player = duel.prompt().map_or(0, Prompt::player);
        measured.observe.push(nanos(|| {
            choices.0.clear();
            duel.observe_into(player, &mut choices)
        }));
        let Some(hash) = hash.filter(|_| check) else { return };
        let view = looks(duel);
        let repeated = match seen.get(&hash) {
            Some((first, first_view)) => {
                measured.repeats += 1;
                consistent(first, first_view, duel, &view).map(|apart| apart.map(|apart| format!("repeated, {apart}")))
            }
            None => {
                seen.insert(hash, (duel.clone(), view.clone()));
                Ok(None)
            }
        };
        // The same with two copies of a card swapped, which should hash apart unless nothing tells them apart; not at an idle or
        // battle prompt, whose lists, found from the board when it opened, would still name the copies as before.
        let commands = matches!(duel.prompt(), Some(Prompt::IdleCommand { .. } | Prompt::BattleCommand { .. }));
        let pair = copies(duel).filter(|_| !commands);
        let swapped = pair.map(|(card, other)| {
            let mut swapped = duel.clone();
            swapped.swap_cards(card, other);
            swapped
        });
        let swap = match swapped {
            Some(swapped) if swapped.state_hash() == Some(hash) => {
                measured.swaps_merged += 1;
                let apart = consistent(duel, &view, &swapped, &looks(&swapped));
                apart.map(|apart| apart.map(|apart| format!("swapped {pair:?}, {apart}")))
            }
            other => {
                measured.swaps_split += u64::from(other.is_some());
                Ok(None)
            }
        };
        match repeated.and_then(|repeated| Ok(repeated.or(swap?))) {
            Ok(None) => {}
            Ok(Some(apart)) => measured.mismatches.push(format!("seed {seed}, {:?}: {apart}", duel.prompt())),
            Err(error) => failure = Some(error.to_string()),
        }
    })?;
    failure.map_or(Ok(()), |failure| Err(failure.into()))
}

fn median(nanos: &mut [u32]) -> u32 {
    nanos.sort_unstable();
    nanos.get(nanos.len() / 2).copied().unwrap_or(0)
}

fn mean(nanos: &[u32]) -> f64 {
    nanos.iter().map(|&nanos| f64::from(nanos)).sum::<f64>() / nanos.len().max(1) as f64
}

fn main() -> BenchResult<()> {
    let [duels, seed, check] = flags(["--duels", "--seed", "--check"], [100, 1, 0])?;
    let (cards, decks) = load_decks(false, true)?;
    let mut measured = Measured::default();
    for seed in seed..seed + duels {
        measure(seed, cards, &decks, check != 0, &mut measured)?;
    }
    let out = &mut std::io::stdout().lock();
    writeln!(out, "{duels} Elfnote mirror duels, decisions only (more than one legal action)")?;
    writeln!(
        out,
        "{:<20}{:>10}{:>10}{:>14}{:>12}",
        "prompt", "decisions", "hashed", "median ns", "mean ns"
    )?;
    let (mut decisions, mut hashed, mut idle) = (0, 0, 0);
    for (name, kind) in &mut measured.kinds {
        decisions += kind.decisions;
        hashed += kind.hashed;
        if name == "IdleCommand" {
            idle += kind.hashed;
        }
        let share = format!("{:.0}%", 100.0 * kind.hashed as f64 / kind.decisions as f64);
        let mean = mean(&kind.nanos);
        writeln!(
            out,
            "{name:<20}{:>10}{share:>10}{:>14}{mean:>12.0}",
            kind.decisions,
            median(&mut kind.nanos)
        )?;
    }
    let percent = |count: u64| 100.0 * count as f64 / decisions as f64;
    writeln!(
        out,
        "hashed: {:.1}% of decisions, {:.1}% at idle prompts alone",
        percent(hashed),
        percent(idle)
    )?;
    let (card_mean, observe_mean) = (mean(&measured.action_card), mean(&measured.observe));
    let (card_median, observe_median) = (median(&mut measured.action_card), median(&mut measured.observe));
    writeln!(
        out,
        "cards of all legal actions per decision: action_card median {card_median} ns, mean {card_mean:.0} ns; \
         observe_into median {observe_median} ns, mean {observe_mean:.0} ns"
    )?;
    if check != 0 {
        writeln!(
            out,
            "repeated hashes: {}; copies swapped: {} hashing alike, {} apart; mismatches: {}",
            measured.repeats,
            measured.swaps_merged,
            measured.swaps_split,
            measured.mismatches.len()
        )?;
        for mismatch in &measured.mismatches {
            writeln!(out, "{}", mismatch.lines().next().unwrap_or_default())?;
        }
        for mismatch in measured.mismatches.iter().take(3) {
            writeln!(out, "{mismatch}")?;
        }
    }
    Ok(())
}
