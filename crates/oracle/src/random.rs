use std::collections::HashSet;

use carddata::CardDatabase;
use ocgcore_sys::{LOCATION_MZONE, LOCATION_SZONE};
use rand::Rng;
use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom, index};

use crate::message::Prompt;

const CANCEL: i32 = -1;
// Monster zone sequence 7 does not exist; see SelectPlace in playerop.cpp.
const ZONES: u32 = 0xff7f_ff7f;

// A random valid response as playerop.cpp parses it (uniform for single choices), None if the prompt admits none.
pub fn respond(prompt: &Prompt, cards: &CardDatabase, rng: &mut impl Rng) -> Option<Vec<u8>> {
    Some(match prompt {
        Prompt::BattleCommand {
            activations,
            attacks,
            to_main2,
            to_end,
            ..
        } => command(&[*activations, *attacks], &[*to_main2, *to_end], rng)?,
        Prompt::IdleCommand {
            summons,
            special_summons,
            repositions,
            monster_sets,
            spell_sets,
            activations,
            to_battle,
            to_end,
            shuffle,
            ..
        } => command(
            &[*summons, *special_summons, *repositions, *monster_sets, *spell_sets, *activations],
            &[*to_battle, *to_end, *shuffle],
            rng,
        )?,
        Prompt::EffectYesNo { .. } | Prompt::YesNo { .. } => ints([rng.random_range(0..=1)]),
        Prompt::Option { options, .. } | Prompt::AnnounceNumber { options, .. } => ints([index_of(options.len(), rng)?]),
        Prompt::Card {
            cancelable,
            min,
            max,
            count,
            ..
        } => {
            if *cancelable && rng.random_ratio(1, max.saturating_sub(*min) + 2) {
                ints([CANCEL])
            } else {
                let size = (*min..=*max).choose(rng)? as usize;
                selection(index::sample(rng, *count, size.min(*count)))
            }
        }
        Prompt::Chain { forced, count, .. } => ints([if *forced {
            index_of(*count, rng)?
        } else {
            rng.random_range(CANCEL..*count as i32)
        }]),
        Prompt::Place { player, count, blocked } => place(*player, *count, *blocked, rng)?,
        Prompt::Position { positions, .. } => ints([*bits(u64::from(*positions & 0xf)).choose(rng)? as i32]),
        Prompt::Tribute {
            cancelable,
            min,
            max,
            releases,
            ..
        } => {
            if *cancelable && rng.random_ratio(1, max.saturating_sub(*min) + 2) {
                ints([CANCEL])
            } else {
                tribute(*min, *max as usize, releases, rng)?
            }
        }
        Prompt::Sort { count, .. } => {
            let mut order: Vec<u8> = (0..*count as u8).collect();
            order.shuffle(rng);
            order
        }
        Prompt::Counter { count, counters, .. } => counter(*count, counters, rng)?,
        Prompt::Sum {
            overflow: true,
            target,
            must,
            params,
            ..
        } => selection(sum_overflow(*target, must, params, rng)?),
        Prompt::Sum {
            target,
            min,
            max,
            must,
            params,
            ..
        } => selection(sum_exact(*target, *min, *max, must, params, rng)?),
        Prompt::UnselectCard {
            finishable,
            cancelable,
            count,
            ..
        } => {
            let first = if *finishable || *cancelable { CANCEL } else { 0 };
            match (first..*count as i32).choose(rng)? {
                CANCEL => ints([CANCEL]),
                index => ints([1, index]),
            }
        }
        Prompt::RockPaperScissors { .. } => ints([rng.random_range(1..=3)]),
        Prompt::AnnounceRace { count, available, .. } => announce(*count, *available, rng)?.to_ne_bytes().to_vec(),
        Prompt::AnnounceAttribute { count, available, .. } => (announce(*count, u64::from(*available), rng)? as u32).to_ne_bytes().to_vec(),
        Prompt::AnnounceCard { opcodes, .. } => ints([cards.cards().filter(|card| card.is_declarable(opcodes)).choose(rng)?.code as i32]),
    })
}

fn ints<const N: usize>(values: [i32; N]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_ne_bytes()).collect()
}

fn index_of(count: usize, rng: &mut impl Rng) -> Option<i32> {
    (count > 0).then(|| rng.random_range(0..count as i32))
}

fn bits(mask: u64) -> Vec<u64> {
    (0..64).map(|bit| 1 << bit).filter(|bit| mask & bit != 0).collect()
}

// Response type 0: a count and u32 indices into the prompt's card list.
fn selection(indices: impl IntoIterator<Item = usize>) -> Vec<u8> {
    let indices: Vec<i32> = indices.into_iter().map(|index| index as i32).collect();
    [0, indices.len() as i32]
        .iter()
        .chain(&indices)
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

// The low u16 picks the list, the high u16 the index in it; the flags follow the lists as further kinds.
fn command(lists: &[usize], flags: &[bool], rng: &mut impl Rng) -> Option<Vec<u8>> {
    let entries = lists
        .iter()
        .enumerate()
        .flat_map(|(kind, &count)| (0..count).map(move |index| (index << 16 | kind) as i32));
    let flags = flags
        .iter()
        .zip(lists.len() as i32..)
        .filter_map(|(&flag, kind)| flag.then_some(kind));
    Some(ints([entries.chain(flags).choose(rng)?]))
}

fn place(player: u8, count: u8, mut blocked: u32, rng: &mut impl Rng) -> Option<Vec<u8>> {
    let mut response = Vec::with_capacity(3 * usize::from(count));
    for _ in 0..count {
        let zone = (0..32).filter(|zone| (ZONES & !blocked) & 1 << zone != 0).choose(rng)?;
        blocked |= 1 << zone;
        let location = if zone % 16 >= 8 { LOCATION_SZONE } else { LOCATION_MZONE };
        response.extend([if zone >= 16 { player ^ 1 } else { player }, location, zone as u8 % 8]);
    }
    Some(response)
}

// Takes cards until their release count reaches `min`, then possibly more up to `max` cards.
fn tribute(min: u32, max: usize, releases: &[u8], rng: &mut impl Rng) -> Option<Vec<u8>> {
    let mut order: Vec<usize> = (0..releases.len()).collect();
    order.shuffle(rng);
    let (mut total, mut needed) = (0, 0);
    while total < min {
        total += u32::from(*releases.get(*order.get(needed)?)?);
        needed += 1;
    }
    let size = (needed..=max.min(order.len())).choose(rng)?;
    Some(selection(order.into_iter().take(size)))
}

fn counter(count: u16, counters: &[u16], rng: &mut impl Rng) -> Option<Vec<u8>> {
    let mut taken = vec![0u16; counters.len()];
    for _ in 0..count {
        let open = taken
            .iter()
            .zip(counters)
            .enumerate()
            .filter(|(_, (taken, available))| taken < available);
        let (index, _) = open.choose(rng)?;
        *taken.get_mut(index)? += 1;
    }
    Some(taken.iter().flat_map(|value| value.to_ne_bytes()).collect())
}

fn sum_values(param: u32) -> impl Iterator<Item = u32> {
    let (value, alternative) = (param & 0xffff, param >> 16);
    std::iter::once(value)
        .chain((alternative != value).then_some(alternative))
        .filter(|&value| value > 0)
}

// Must cards come first in the core's check and every value is positive, so any order of the chosen cards passes.
fn sum_exact(target: u32, min: u32, max: u32, must: &[u32], params: &[u32], rng: &mut impl Rng) -> Option<Vec<usize>> {
    let mut remainders = HashSet::from([target]);
    for &param in must {
        remainders = remainders
            .iter()
            .flat_map(|&left| sum_values(param).filter_map(move |value| left.checked_sub(value)))
            .collect();
    }
    let mut order: Vec<usize> = (0..params.len()).collect();
    order.shuffle(rng);
    let mut search = SumSearch {
        params,
        order,
        min: min as usize,
        max: max as usize,
        failed: HashSet::new(),
        chosen: Vec::new(),
    };
    let mut remainders: Vec<u32> = remainders.into_iter().collect();
    remainders.sort_unstable();
    remainders.shuffle(rng);
    remainders
        .into_iter()
        .find(|&remainder| search.find(0, remainder, rng))
        .map(|_| search.chosen)
}

struct SumSearch<'a> {
    params: &'a [u32],
    order: Vec<usize>,
    min: usize,
    max: usize,
    failed: HashSet<(usize, usize, u32)>,
    chosen: Vec<usize>,
}

impl SumSearch<'_> {
    fn find(&mut self, position: usize, remainder: u32, rng: &mut impl Rng) -> bool {
        if remainder == 0 {
            return self.chosen.len() >= self.min;
        }
        let Some(&index) = self.order.get(position) else { return false };
        if self.chosen.len() == self.max || self.failed.contains(&(position, self.chosen.len(), remainder)) {
            return false;
        }
        let mut values: Vec<u32> = self.params.get(index).map(|&param| sum_values(param).collect()).unwrap_or_default();
        values.shuffle(rng);
        let include_first = rng.random_bool(0.5);
        for include in [include_first, !include_first] {
            if include {
                for &value in values.iter().filter(|&&value| value <= remainder) {
                    self.chosen.push(index);
                    if self.find(position + 1, remainder - value, rng) {
                        return true;
                    }
                    self.chosen.pop();
                }
            } else if self.find(position + 1, remainder, rng) {
                return true;
            }
        }
        self.failed.insert((position, self.chosen.len(), remainder));
        false
    }
}

// The core accepts a selection whose largest total reaches the target while dropping its smallest card would not.
fn sum_overflow(target: u32, must: &[u32], params: &[u32], rng: &mut impl Rng) -> Option<Vec<usize>> {
    let bounds = |param: u32| {
        let (value, alternative) = (param & 0xffff, param >> 16);
        (
            if alternative != 0 && alternative < value {
                alternative
            } else {
                value
            },
            value.max(alternative),
        )
    };
    let mut order: Vec<usize> = (0..params.len()).collect();
    order.shuffle(rng);
    let mut chosen: Vec<(usize, u32, u32)> = Vec::new();
    let totals = |chosen: &[(usize, u32, u32)]| {
        let all = must
            .iter()
            .map(|&param| bounds(param))
            .chain(chosen.iter().map(|&(_, low, high)| (low, high)));
        all.fold((0, 0, u32::MAX), |(low_sum, high_sum, smallest), (low, high)| {
            (low_sum + low, high_sum + high, smallest.min(low))
        })
    };
    for index in order {
        if totals(&chosen).1 >= target {
            break;
        }
        let (low, high) = bounds(*params.get(index)?);
        chosen.push((index, low, high));
    }
    chosen.sort_by_key(|&(_, low, _)| std::cmp::Reverse(low));
    loop {
        let (low_sum, high_sum, smallest) = totals(&chosen);
        if high_sum < target {
            return None;
        }
        if low_sum.saturating_sub(smallest) < target {
            return Some(chosen.into_iter().map(|(index, ..)| index).collect());
        }
        chosen.pop()?;
    }
}

fn announce(count: u8, available: u64, rng: &mut impl Rng) -> Option<u64> {
    let options = bits(available);
    (options.len() >= usize::from(count)).then(|| options.choose_multiple(rng, usize::from(count)).fold(0, |mask, bit| mask | bit))
}
