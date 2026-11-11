// How a build plays against another from the same seeds. `record SET DUELS SEED FILE` plays uniform random duels and logs
// each choice by what it picks (stable across builds, unlike its list index) with a hash of the state after it;
// `replay SET FILE` plays the log on this build and prints each duel's first divergence as `seed index kind`;
// `dump SET FILE DIVERGENCES` prints the choices and the state after each listed action, to compare two builds' dumps;
// `stream SET FILE SEED OUT` writes this build's message stream of one logged duel, EDOPro's on a build without `fast`.
// SET is normal, classic or elfnote (the bench's deck sets), or cards (the diff's scenarios, SCENARIOS_PER_CARD per ported card).
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufWriter, Write};

use bench::{BenchResult, MAX_ACTIONS, load_decks, setup};
use carddata::{CardDatabase, Decks};
use diff::Vendor;
use engine::{Action, BattleCommand, Choice, Duel, FAST, IdleCommand, Status};
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rayon::prelude::*;

type Log = Vec<(u64, Vec<u32>)>;

const SCENARIOS_PER_CARD: u64 = 5;

enum Duels {
    Decks(&'static CardDatabase, Decks),
    Scenarios(&'static Vendor),
}

impl Duels {
    fn load(set: &str) -> BenchResult<Self> {
        let decks = |effects, elfnote| load_decks(effects, elfnote).map(|(cards, decks)| Self::Decks(cards, decks));
        match set {
            "normal" => decks(false, false),
            "classic" => decks(true, false),
            "elfnote" => decks(false, true),
            "cards" => Ok(Self::Scenarios(Vendor::leak(Vendor::scenarios)?)),
            other => Err(format!("unknown deck set {other}").into()),
        }
    }

    // A deck set's duel `seed`, or scenario seed % SCENARIOS_PER_CARD of the (seed / SCENARIOS_PER_CARD)-th ported card.
    fn start(&self, seed: u64) -> BenchResult<(Duel, StdRng)> {
        match self {
            Self::Decks(cards, decks) => Ok(setup(seed, cards, decks)?),
            Self::Scenarios(vendor) => {
                let card = scripts::CARD_SCRIPTS
                    .get(usize::try_from(seed / SCENARIOS_PER_CARD)?)
                    .ok_or("no such card")?;
                Ok(diff::scenario_duel(vendor, card.code, (seed % SCENARIOS_PER_CARD) as u32)?)
            }
        }
    }
}

fn hash(value: &impl Hash) -> u32 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as u32
}

fn state_hash(duel: &Duel) -> u32 {
    hash(&(duel.state(), duel.result()))
}

// The action without its index into this build's lists, which may be ordered differently in the other.
fn without_index(action: Action) -> Action {
    match action {
        Action::IdleCommand(command) => Action::IdleCommand(match command {
            IdleCommand::Summon(_) => IdleCommand::Summon(0),
            IdleCommand::SpecialSummon(_) => IdleCommand::SpecialSummon(0),
            IdleCommand::Reposition(_) => IdleCommand::Reposition(0),
            IdleCommand::MonsterSet(_) => IdleCommand::MonsterSet(0),
            IdleCommand::SpellSet(_) => IdleCommand::SpellSet(0),
            IdleCommand::Activate(_) => IdleCommand::Activate(0),
            other => other,
        }),
        Action::BattleCommand(BattleCommand::Activate(_)) => Action::BattleCommand(BattleCommand::Activate(0)),
        Action::BattleCommand(BattleCommand::Attack(_)) => Action::BattleCommand(BattleCommand::Attack(0)),
        Action::Option(_) => Action::Option(0),
        Action::Chain(Some(_)) => Action::Chain(Some(0)),
        Action::Card(_) => Action::Card(0),
        other => other,
    }
}

// The prompt's choices keyed by what they pick: the card and description, numbered among choices that pick the same.
fn keyed(duel: &Duel) -> Vec<(Choice, u32)> {
    let Some(prompt) = duel.prompt() else { return Vec::new() };
    let mut seen: HashMap<u32, u32> = HashMap::new();
    let choices = duel.observe(prompt.player()).choices.into_iter();
    choices
        .map(|choice| {
            let same = seen
                .entry(hash(&(without_index(choice.action), choice.card, choice.description)))
                .or_default();
            *same += 1;
            (
                choice,
                hash(&(without_index(choice.action), choice.card, choice.description, *same)),
            )
        })
        .collect()
}

// Pairs of (choice key, state hash after it).
fn record(seed: u64, duels: &Duels) -> BenchResult<Vec<u32>> {
    let (mut duel, mut rng) = duels.start(seed)?;
    let mut log = Vec::new();
    let mut status = duel.process();
    while status == Status::Awaiting && (log.len() as u64) < 2 * MAX_ACTIONS {
        let &(choice, key) = keyed(&duel).choose(&mut rng).ok_or("no legal action")?;
        status = duel.apply(choice.action)?;
        log.extend([key, state_hash(&duel)]);
    }
    Ok(log)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Divergence {
    NoMatchingOption,
    State,
    Longer,
    Error,
}

// Plays the log from the duel's start up to its first divergence; `on_action` sees each action's choices, the key
// chosen and the duel after it (None without a matching choice), and stops the replay by returning false.
fn replay(
    duel: &mut Duel,
    log: &[u32],
    mut on_action: impl FnMut(usize, &[(Choice, u32)], u32, Option<&Duel>) -> bool,
) -> BenchResult<Option<(usize, Divergence)>> {
    duel.process();
    for (index, &[key, state]) in log.as_chunks::<2>().0.iter().enumerate() {
        let choices = keyed(duel);
        let Some(&(choice, _)) = choices.iter().find(|&&(_, other)| other == key) else {
            on_action(index, &choices, key, None);
            return Ok(Some((index, Divergence::NoMatchingOption)));
        };
        duel.apply(choice.action)?;
        if !on_action(index, &choices, key, Some(duel)) {
            return Ok(None);
        }
        if state_hash(duel) != state {
            return Ok(Some((index, Divergence::State)));
        }
    }
    Ok(duel.prompt().is_some().then_some((log.len() / 2, Divergence::Longer)))
}

fn read_log(path: &str) -> BenchResult<Log> {
    let bytes = std::fs::read(path)?;
    let mut words = bytes.as_chunks::<4>().0.iter().map(|&word| u32::from_le_bytes(word));
    let mut log = Vec::new();
    while let (Some(low), Some(high), Some(len)) = (words.next(), words.next(), words.next()) {
        log.push((u64::from(high) << 32 | u64::from(low), words.by_ref().take(len as usize).collect()));
    }
    Ok(log)
}

fn write_log(path: &str, log: &Log) -> BenchResult<()> {
    let mut out = BufWriter::new(std::fs::File::create(path)?);
    for (seed, words) in log {
        let header = [*seed as u32, (*seed >> 32) as u32, u32::try_from(words.len())?];
        for word in header.iter().chain(words) {
            out.write_all(&word.to_le_bytes())?;
        }
    }
    Ok(out.flush()?)
}

fn dump(out: &mut impl Write, choices: &[(Choice, u32)], key: u32, after: Option<&Duel>) -> std::io::Result<()> {
    for (choice, other) in choices {
        let mark = if *other == key { '*' } else { ' ' };
        writeln!(
            out,
            "choice{mark} {other:08x} {:?} {:?} {:#x}",
            choice.action, choice.card, choice.description
        )?;
    }
    if let Some(duel) = after {
        let state = duel.state();
        writeln!(
            out,
            "after lp {:?} turn {} phase {:#x} chain {}",
            state.lp, state.turn, state.phase, state.chain
        )?;
        for card in state.cards {
            writeln!(out, "card {card:?}")?;
        }
    }
    Ok(())
}

fn main() -> BenchResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |index: usize| {
        args.get(index)
            .map(String::as_str)
            .ok_or("usage: see the comment atop divergence.rs")
    };
    let duels = &Duels::load(arg(1)?)?;
    let out = &mut std::io::stdout().lock();
    match arg(0)? {
        "record" => {
            let (count, seed): (u64, u64) = (arg(2)?.parse()?, arg(3)?.parse()?);
            let recorded: Vec<(u64, BenchResult<Vec<u32>>)> = (seed..seed + count)
                .into_par_iter()
                .map(|seed| (seed, record(seed, duels)))
                .collect();
            let failed = recorded.iter().filter(|(_, words)| words.is_err()).count();
            let log: Log = recorded.into_iter().filter_map(|(seed, words)| Some((seed, words.ok()?))).collect();
            write_log(arg(4)?, &log)?;
            let actions: usize = log.iter().map(|(_, words)| words.len() / 2).sum();
            writeln!(
                out,
                "{count} duels ({failed} failed to set up or play), {actions} actions, fast {FAST}"
            )?;
        }
        "replay" => {
            let log = read_log(arg(2)?)?;
            let results: Vec<_> = log
                .par_iter()
                .map(|(seed, words)| {
                    let divergence = duels
                        .start(*seed)
                        .and_then(|(mut duel, _)| replay(&mut duel, words, |_, _, _, _| true));
                    (*seed, divergence.unwrap_or(Some((0, Divergence::Error))))
                })
                .collect();
            let mut kinds: HashMap<Divergence, usize> = HashMap::new();
            for (seed, divergence) in &results {
                if let Some((index, kind)) = divergence {
                    writeln!(out, "{seed} {index} {kind:?}")?;
                    *kinds.entry(*kind).or_default() += 1;
                }
            }
            writeln!(
                out,
                "# {} duels, {} diverged: {kinds:?}",
                results.len(),
                kinds.values().sum::<usize>()
            )?;
        }
        "dump" => {
            let log: HashMap<u64, Vec<u32>> = read_log(arg(2)?)?.into_iter().collect();
            for line in std::fs::read_to_string(arg(3)?)?.lines().filter(|line| !line.starts_with('#')) {
                let mut fields = line.split_whitespace();
                let (seed, at): (u64, usize) = (fields.next().ok_or("seed")?.parse()?, fields.next().ok_or("index")?.parse()?);
                let words = log.get(&seed).ok_or("no such duel in the log")?;
                writeln!(out, "duel {seed} action {at}")?;
                let mut written = Ok(());
                let (mut duel, _) = duels.start(seed)?;
                let divergence = replay(&mut duel, words, |index, choices, key, after| {
                    if index == at {
                        written = dump(out, choices, key, after);
                    }
                    index < at
                })?;
                written?;
                writeln!(out, "divergence {divergence:?}")?;
            }
        }
        "stream" => {
            let seed: u64 = arg(3)?.parse()?;
            let (_, words) = read_log(arg(2)?)?
                .into_iter()
                .find(|&(other, _)| other == seed)
                .ok_or("no such duel in the log")?;
            let (mut duel, _) = duels.start(seed)?;
            duel.set_trace(true);
            let divergence = replay(&mut duel, &words, |_, _, _, _| true)?;
            std::fs::write(arg(4)?, duel.take_trace())?;
            writeln!(out, "divergence {divergence:?}")?;
        }
        other => return Err(format!("unknown mode {other}").into()),
    }
    Ok(())
}
