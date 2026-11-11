use std::collections::BTreeMap;
use std::io::Write;
use std::sync::OnceLock;

use diff::{Vendor, diff_scenario, each_state, records, scenario_records};
use engine::{Action, Duel, DuelRecord, Observation, Prompt, SeenCard};
use fiber::Fiber;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use scripts::LIBRARY;
use sim::*;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Elfnote mirror records and one scenario per ported card, built once for the tests that share them.
fn all_records() -> Result<&'static [(&'static Vendor, DuelRecord)]> {
    static RECORDS: OnceLock<std::result::Result<Vec<(&'static Vendor, DuelRecord)>, String>> = OnceLock::new();
    let build = || -> Result<Vec<_>> {
        let (elfnote, scenarios) = (Vendor::leak(Vendor::elfnote)?, Vendor::leak(Vendor::scenarios)?);
        let elfnote_records = records(elfnote, 0..40)?.into_iter().map(|record| (elfnote, record));
        let scenario_records = scenario_records(scenarios)?.into_iter().map(|record| (scenarios, record));
        Ok(elfnote_records.chain(scenario_records).collect())
    };
    let records = RECORDS.get_or_init(|| build().map_err(|error| error.to_string()));
    Ok(records.as_ref().map_err(String::clone)?)
}

// Legal actions get distinct indices that decode back, the mask holds them and the choices line up; tracks longest lists.
fn check_encoding(duel: &Duel, index: usize, longest: &mut BTreeMap<Section, usize>) -> Result {
    let legal = duel.policy_actions();
    let indices: Vec<usize> = indices(&legal).collect::<std::result::Result<_, _>>()?;
    let mut mask = vec![false; ACTIONS];
    write_mask(&legal, &mut mask)?;
    let mut distinct = indices.clone();
    distinct.sort_unstable();
    distinct.dedup();
    if distinct.len() != legal.len() || mask.iter().filter(|&&set| set).count() != legal.len() {
        return Err(format!("{legal:?} encode to {indices:?}").into());
    }
    for (&action, &at) in legal.iter().zip(&indices) {
        let round_trip = if legal.len() > 64 {
            find(&legal, at)
        } else {
            encode(duel, action).and_then(|at| decode(duel, at))
        };
        if round_trip != Some(action) || mask.get(at) != Some(&true) {
            return Err(format!("{action:?} at {at} decodes to {round_trip:?}").into());
        }
    }
    for (section, slot) in slots(&legal) {
        let longest = longest.entry(section).or_default();
        *longest = (*longest).max(slot + 1);
    }
    if let Some(prompt) = duel.prompt().filter(|_| index.is_multiple_of(25)) {
        let choices = duel.observe(prompt.player()).choices;
        let aligned: Vec<usize> = sim::indices(choices.iter().map(|choice| &choice.action)).collect::<std::result::Result<_, _>>()?;
        if aligned != indices {
            return Err("the choices do not line up with the legal actions".into());
        }
    }
    Ok(())
}

// The tensors hold the observation's cards in order and each legal action's section and card; returns the card and link counts.
fn check_tensors(observation: &Observation) -> Result<[usize; 2]> {
    let mut buffers = Buffers::new(1);
    buffers.tensors()?.write(0, Some(observation))?;
    let [_, cards, _, actions] = &buffers.0;
    let (cards, _) = cards.as_chunks::<{ CARD_FEATURES.len() }>();
    let found = cards.iter().filter(|&&[valid, ..]| valid == 1);
    let found = found.map(|&[_, _, location, sequence, position, code, ..]| [location, sequence, position, code]);
    let place = |card: &SeenCard| [card.location, card.sequence, card.position].map(i32::from);
    let expected = observation.cards.iter().map(|card| {
        let [location, sequence, position] = place(card);
        [location, sequence, position, card.code as i32]
    });
    if !found.eq(expected) {
        return Err("the card table differs from the observation".into());
    }
    let (actions, _) = actions.as_chunks::<{ ACTION_FEATURES.len() }>();
    let legal = || observation.choices.iter().map(|choice| &choice.action);
    for ((choice, (section, _)), index) in observation.choices.iter().zip(slots(legal())).zip(indices(legal())) {
        let &[found_section, row, code, ..] = actions.get(index?).ok_or("no action row")?;
        let card = usize::try_from(row).ok().and_then(|row| observation.cards.get(row));
        let same_card = card.map(place) == choice.card.as_ref().map(place);
        if found_section != section as i32 || !same_card || choice.card.is_some_and(|card| card.code as i32 != code) {
            return Err(format!("{choice:?} encodes to {:?}", actions.get(index?)).into());
        }
    }
    if actions.iter().filter(|&&[section, ..]| section >= 0).count() != observation.choices.len() {
        return Err("rows for actions that are not legal".into());
    }
    Ok([observation.cards.len(), observation.chain.len()])
}

#[test]
fn should_encode_every_legal_action_distinctly_within_the_caps() {
    let records = all_records().unwrap();
    let checked = records.par_iter().map(|(vendor, record)| {
        let (mut longest, mut most) = (BTreeMap::new(), [0; 2]);
        each_state(vendor, record, |duel, index| {
            check_encoding(duel, index, &mut longest)?;
            for player in (0..2).filter(|_| index.is_multiple_of(25)) {
                let counts = check_tensors(&duel.observe(player))?;
                most = [0, 1].map(|at| most.get(at).max(counts.get(at)).copied().unwrap_or(0));
            }
            Ok(())
        })?;
        Ok((longest, most))
    });
    let checked: Vec<(BTreeMap<Section, usize>, [usize; 2])> = checked.collect::<Result<_>>().unwrap();
    let mut longest = BTreeMap::new();
    for (section, length) in checked.iter().flat_map(|(longest, _)| longest) {
        let entry = longest.entry(*section).or_insert(0);
        *entry = (*length).max(*entry);
    }
    let [cards, chain] = [0, 1].map(|at| checked.iter().filter_map(|(_, most)| most.get(at)).max().copied().unwrap_or(0));
    let mut out = std::io::stdout();
    writeln!(out, "{ACTIONS} actions; longest slot range per section {longest:?}").unwrap();
    writeln!(
        out,
        "most cards observed {cards} (cap {MAX_OBS_CARDS}), longest chain {chain} (cap {MAX_CHAIN})"
    )
    .unwrap();
}

#[test]
fn should_encode_a_duel_straight_as_its_observation() {
    let checked = all_records().unwrap().par_iter().map(|(vendor, record)| {
        each_state(vendor, record, |duel, _| {
            for player in 0..2 {
                let (mut observed, mut straight) = (Buffers::new(1), Buffers::new(1));
                observed.tensors()?.write(0, Some(&duel.observe(player)))?;
                straight.tensors()?.write_duel(0, duel, player)?;
                if observed != straight {
                    return Err(format!("player {player}: the duel encodes differently from its observation").into());
                }
            }
            Ok(())
        })
    });
    checked.collect::<Result<()>>().unwrap();
}

#[test]
fn should_reject_an_action_beyond_its_section() {
    let overflow = Err(EncodingError::Overflow {
        section: Section::Card,
        slot: Section::Card.slots(),
    });
    let legal = [Action::Finish, Action::Card(Section::Card.slots() as u16)];
    assert_eq!(write_mask(&legal, &mut [false; ACTIONS]), overflow.map(drop));
    assert_eq!(indices(&legal).nth(1), Some(overflow));
}

#[test]
fn should_offer_the_black_goat_laughs_at_most_the_cap_of_declarable_codes() {
    let scenarios = Vendor::leak(Vendor::scenarios).unwrap();
    let record = diff_scenario(scenarios, 49299410, 0).unwrap().record;
    let mut declarations = 0;
    each_state(scenarios, &record, |duel, _| {
        let Some(Prompt::AnnounceCard { .. }) = duel.prompt() else {
            return Ok(());
        };
        let (offered, legal) = (duel.policy_actions(), duel.legal_actions());
        let declarable = offered
            .iter()
            .all(|action| matches!(action, Action::Code(_)) && legal.contains(action));
        if !declarable || offered.len() > engine::MAX_DECLARED_CODES || legal.len() <= offered.len() {
            return Err(format!("{} legal codes, offered {offered:?}", legal.len()).into());
        }
        duel.clone().apply(*offered.first().ok_or("no code offered")?)?;
        declarations += 1;
        Ok(())
    })
    .unwrap();
    assert!(declarations > 0);
}

// Auto-advance applies exactly the single-option prompts a step-by-step walk sees (the acting player's under Own).
fn check_auto_advance(duel: &Duel) -> Result {
    let Some(player) = duel.prompt().map(Prompt::player) else {
        return Ok(());
    };
    for mode in [AutoAdvance::Own, AutoAdvance::Both] {
        let (mut advanced, mut walked) = (Fiber::new(duel.clone()), duel.clone());
        let mut legal = Vec::new();
        let skipped = auto_advance(&mut advanced, mode, player, &mut legal)?;
        let skippable = |duel: &Duel| {
            let own = duel
                .prompt()
                .is_some_and(|prompt| mode == AutoAdvance::Both || prompt.player() == player);
            own && duel.policy_actions().len() == 1
        };
        for _ in 0..skipped {
            if !skippable(&walked) {
                return Err(format!("{mode:?} skipped {:?} with {:?}", walked.prompt(), walked.policy_actions()).into());
            }
            walked.apply(walked.policy_actions().into_iter().next().ok_or("no action")?)?;
        }
        let same = (0..2).all(|player| advanced.observe(player) == walked.observe(player));
        if skippable(&walked) || !same || legal != walked.policy_actions() {
            return Err(format!("{mode:?} stopped after {skipped} at {:?}", walked.prompt()).into());
        }
    }
    Ok(())
}

#[test]
fn should_auto_advance_only_single_option_prompts() {
    let checked = all_records().unwrap().par_iter().try_for_each(|(vendor, record)| {
        each_state(vendor, record, |duel, index| {
            if index.is_multiple_of(10) {
                check_auto_advance(duel)
            } else {
                Ok(())
            }
        })
    });
    checked.unwrap();
}

const SLOTS: usize = 8;
const MAX_STEPS: u64 = 1500;

// The `k`-th of the legal indices in increasing order, k drawn uniformly.
fn pick(indices: impl Iterator<Item = usize>, count: usize, rng: &mut StdRng) -> usize {
    let mut indices: Vec<usize> = indices.collect();
    indices.sort_unstable();
    let k = rng.random_range(0..count.max(1));
    indices.get(k).copied().unwrap_or(0)
}

type Generated = std::result::Result<Option<Duel>, engine::PositionError>;

fn alone(generator: impl Fn(usize, u64) -> Generated, mode: AutoAdvance) -> Result<Vec<Outcome>> {
    let mut outcomes = Vec::new();
    for slot in 0..SLOTS {
        let mut rng = StdRng::seed_from_u64(slot as u64);
        for episode in 0.. {
            let Some(duel) = generator(slot, episode)? else { break };
            let mut duel = Fiber::new(duel);
            duel.process();
            let mut legal = Vec::new();
            let first = duel.prompt().map_or(0, Prompt::player);
            let mut skipped = auto_advance(&mut duel, mode, first, &mut legal)?;
            let mut steps = 0;
            while !legal.is_empty() && steps < MAX_STEPS {
                let index = pick(indices(&legal).flatten(), legal.len(), &mut rng);
                let player = duel.prompt().map_or(0, Prompt::player);
                duel.apply(find(&legal, index).ok_or("no action")?)?;
                skipped += auto_advance(&mut duel, mode, player, &mut legal)?;
                steps += 1;
            }
            let result = duel.result();
            outcomes.push(Outcome {
                slot,
                episode,
                result,
                steps,
                skipped,
            });
        }
    }
    Ok(outcomes)
}

fn replayed(vendor: &'static Vendor, records: &[DuelRecord]) -> impl Fn(usize, u64) -> Generated + Sync + Copy {
    move |slot: usize, episode: u64| {
        let record = records.get(slot + SLOTS * episode as usize);
        record
            .map(|record| Duel::from_position(&record.position, &vendor.cards, &LIBRARY, record.seed))
            .transpose()
    }
}

#[test]
fn should_observe_each_slot_for_its_acting_player() {
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    let records = records(vendor, 100..100 + SLOTS as u64).unwrap();
    let mut batch = Batch::new(SLOTS, Options::default(), replayed(vendor, &records)).unwrap();
    for _ in 0..200 {
        let actions: Vec<usize> = (0..SLOTS)
            .map(|slot| indices(batch.legal(slot)).flatten().last().unwrap_or(0))
            .collect();
        batch.step(&actions).unwrap();
    }
    let (mut batched, mut single) = (Buffers::new(SLOTS), Buffers::new(SLOTS));
    batch.observe(&mut batched.tensors().unwrap()).unwrap();
    let mut single_tensors = single.tensors().unwrap();
    for slot in 0..SLOTS {
        let observation = batch.duel(slot).and_then(|duel| Some(duel.observe(duel.prompt()?.player())));
        single_tensors.write(slot, observation.as_ref()).unwrap();
    }
    assert_eq!(batched, single);
}

#[test]
fn should_step_a_batch_like_each_duel_alone() {
    let vendor = Vendor::leak(Vendor::elfnote).unwrap();
    let records = records(vendor, 100..124).unwrap();
    let generator = replayed(vendor, &records);
    for auto_advance in [AutoAdvance::Off, AutoAdvance::Own, AutoAdvance::Both] {
        let options = Options {
            auto_advance,
            max_steps: MAX_STEPS,
        };
        let mut batch = Batch::new(SLOTS, options, generator).unwrap();
        let mut rngs: Vec<StdRng> = (0..SLOTS as u64).map(StdRng::seed_from_u64).collect();
        let mut outcomes = batch
            .run(|batch, actions| {
                for ((slot, action), rng) in actions.iter_mut().enumerate().zip(&mut rngs) {
                    let mask = batch.masks().chunks(ACTIONS).nth(slot).unwrap();
                    let count = batch.legal(slot).len();
                    if count > 0 {
                        *action = pick(mask.iter().enumerate().filter(|(_, set)| **set).map(|(index, _)| index), count, rng);
                    }
                }
            })
            .unwrap();
        outcomes.sort_by_key(|outcome| (outcome.slot, outcome.episode));
        assert_eq!(outcomes, alone(generator, auto_advance).unwrap(), "{auto_advance:?}");
        assert!(outcomes.iter().any(|outcome| outcome.result.is_some()));
    }
}
