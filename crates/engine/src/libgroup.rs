// Port of libgroup.cpp: groups are CardSet values, so Group.CreateGroup is CardSet::default() and `#g` is len().
use crate::card::{CardId, CardSet};
use crate::constants::*;
use crate::duel::Duel;
use crate::list::List;
use crate::playerop::sum_halves;
use crate::processor::{SelectSum, SelectUnselectCard};
use crate::script::Script;

impl CardSet {
    pub fn get_first(&self) -> Option<CardId> {
        self.iter().next()
    }

    pub fn merge(&mut self, other: CardSet) {
        *self = self.union(other);
    }

    pub fn is_contains(&self, card: CardId) -> bool {
        self.contains(card)
    }

    pub fn filter(&self, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool, exception: Option<CardSet>) -> CardSet {
        let exception = exception.unwrap_or_default();
        self.iter()
            .filter(|&card| !exception.contains(card) && filter(card, duel))
            .collect()
    }

    pub fn is_exists(&self, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool, count: usize, exception: Option<CardSet>) -> bool {
        let exception = exception.unwrap_or_default();
        self.iter()
            .filter(|&card| !exception.contains(card) && filter(card, duel))
            .take(count)
            .count()
            >= count
    }
}

impl CardSet {
    // Group.Split: the cards the filter holds for, and the others, the exception among them.
    pub fn split(&self, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool, exception: Option<CardSet>) -> (CardSet, CardSet) {
        let exception = exception.unwrap_or_default();
        let matching = self.filter(duel, filter, Some(exception));
        (matching, self.difference(matching))
    }

    pub fn filter_count(&self, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool, exception: Option<CardSet>) -> usize {
        self.filter(duel, filter, exception).len()
    }

    // Lua returns nil for a canceled cancelable selection; the empty group stands for it.
    pub fn select(&self, duel: &mut Duel, player: u8, min: u8, max: u8, exception: Option<CardSet>) -> Script<CardSet> {
        self.select_full(duel, player, min, max, false, exception)
    }

    // A canceled selection is the empty group, where Lua gets nil.
    pub fn select_full(&self, duel: &mut Duel, player: u8, min: u8, max: u8, cancel: bool, exception: Option<CardSet>) -> Script<CardSet> {
        if player > 1 {
            return Ok(CardSet::default());
        }
        let cards = self.difference(exception.unwrap_or_default());
        duel.select_cards(player, cards, cancel, min, max)?;
        Ok(duel.returned_cards())
    }

    pub fn filter_select(
        &self,
        duel: &mut Duel,
        player: u8,
        filter: impl Fn(CardId, &Duel) -> bool,
        min: u8,
        max: u8,
        exception: Option<CardSet>,
    ) -> Script<CardSet> {
        self.filter_select_full(duel, player, filter, min, max, false, exception)
    }

    pub fn filter_select_full(
        &self,
        duel: &mut Duel,
        player: u8,
        filter: impl Fn(CardId, &Duel) -> bool,
        min: u8,
        max: u8,
        cancel: bool,
        exception: Option<CardSet>,
    ) -> Script<CardSet> {
        if player > 1 {
            return Ok(CardSet::default());
        }
        let cards = self.filter(duel, filter, exception);
        duel.select_cards(player, cards, cancel, min, max)?;
        Ok(duel.returned_cards())
    }

    // One card per call, picked from `self` or dropped from `unselect`; None when the selection ends.
    pub fn select_unselect(
        &self,
        duel: &mut Duel,
        unselect: Option<CardSet>,
        player: u8,
        finishable: bool,
        cancelable: bool,
        min: u8,
        max: u8,
    ) -> Script<Option<CardId>> {
        let unselect = unselect.unwrap_or_default();
        if player > 1 || self.iter().any(|card| unselect.contains(card)) {
            return Ok(None);
        }
        duel.core.select_cards = self.iter().collect();
        duel.core.unselect_cards = unselect.iter().collect();
        duel.emplace_process(SelectUnselectCard {
            playerid: player,
            cancelable,
            min: min.min(max),
            max,
            finishable,
            ..SelectUnselectCard::default()
        });
        duel.yield_script()?;
        Ok(match duel.return_cards.canceled {
            true => None,
            false => duel.return_cards.list.first().copied(),
        })
    }

    pub fn random_select(&self, duel: &mut Duel, player: u8, count: usize) -> CardSet {
        let count = count.min(self.len());
        if count == 0 {
            return CardSet::default();
        }
        let mut selected = CardSet::default();
        if count == self.len() {
            selected = *self;
        } else {
            while selected.len() < count {
                let index = duel.random.get_next_integer(0, self.len() as i32 - 1);
                if let Some(card) = self.iter().nth(index as usize) {
                    selected.insert(card);
                }
            }
        }
        let locations: Vec<_> = selected.iter().map(|card| duel.cards.get_info_location(card)).collect();
        let mut message = duel.trace.message(MSG_RANDOM_SELECTED);
        message.u8(player).u32(count as u32);
        for location in locations {
            message.loc(location);
        }
        selected
    }
}

impl CardSet {
    pub fn includes(&self, other: CardSet) -> bool {
        other.difference(*self).is_empty()
    }

    // Group.Match: keeps the cards that pass `filter`, without the exception.
    pub fn keep_matching(&mut self, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool, exception: Option<CardSet>) {
        *self = self.filter(duel, filter, exception);
    }

    // Group.Remove, which drops the cards that pass `filter` other than the exception (`remove` is Group.RemoveCard).
    pub fn remove_matching(&mut self, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool, exception: Option<CardSet>) {
        let exception = exception.unwrap_or_default();
        *self = self.filter(duel, |card, duel| exception.contains(card) || !filter(card, duel), None);
    }

    pub fn get_class_count(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> i64) -> usize {
        let mut classes: crate::list::List<i64, 16> = self.iter().map(|card| value(card, duel)).collect();
        classes.sort_unstable();
        classes.chunk_by(|a, b| a == b).count()
    }

    pub fn get_bitwise_or(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> u64) -> u64 {
        self.iter().fold(0, |total, card| total | value(card, duel))
    }

    // Group.GetMaxGroup: the cards with the highest value and that value, None for an empty group.
    pub fn get_max_group(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> i64) -> Option<(CardSet, i64)> {
        let mut max: Option<(CardSet, i64)> = None;
        for card in self.iter() {
            let op = value(card, duel);
            match &mut max {
                Some((group, max)) if op == *max => group.insert(card),
                Some((_, max)) if op < *max => {}
                _ => max = Some((CardSet::from(card), op)),
            }
        }
        max
    }

    pub fn get_sum(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> i64) -> i64 {
        self.iter().map(|card| value(card, duel)).sum()
    }

    // Group.CheckWithSumEqual: whether the cards Duel.SetSelectedCard set and `min` to `max` of the others add up to
    // `acc`, each counting the low or the high half of its value.
    pub fn check_with_sum_equal(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> u32, acc: u32, min: usize, max: usize) -> bool {
        let (values, must) = self.sum_values(duel, value);
        check_with_sum_limit_m(&values, acc as i32, 0, min, max.max(min), must)
    }

    // Group.CheckWithSumGreater: whether the cards Duel.SetSelectedCard set and some of the others reach `acc`, and
    // would fall short without their smallest value.
    pub fn check_with_sum_greater(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> u32, acc: u32) -> bool {
        let (values, must) = self.sum_values(duel, value);
        check_with_sum_greater_limit_m(&values, acc as i32, 0, 0xffff, must)
    }

    // The values of the must cards, which it clears, then of the others, and how many are must cards.
    fn sum_values(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> u32) -> (List<u32, 16>, usize) {
        let must = duel.core.must_select_cards.take();
        let cards = must.iter().chain(self.difference(must).iter());
        (cards.map(|card| value(card, duel)).collect(), must.len())
    }

    // Group.SelectWithSumEqual; the empty group when no selection adds up (and for a player above 1, where Lua gets
    // nothing).
    pub fn select_with_sum_equal(
        &self,
        duel: &mut Duel,
        player: u8,
        value: impl Fn(CardId, &Duel) -> u32,
        acc: u32,
        min: i32,
        max: i32,
    ) -> Script<CardSet> {
        let min = min.max(0);
        let max = max.max(min);
        let check = |values: &[u32], must| check_with_sum_limit_m(values, acc as i32, 0, min as usize, max as usize, must);
        self.select_with_sum(duel, player, value, acc, min, max, check)
    }

    pub fn select_with_sum_greater(&self, duel: &mut Duel, player: u8, value: impl Fn(CardId, &Duel) -> u32, acc: u32) -> Script<CardSet> {
        let check = |values: &[u32], must| check_with_sum_greater_limit_m(values, acc as i32, 0, 0xffff, must);
        self.select_with_sum(duel, player, value, acc, 0, 0, check)
    }

    fn select_with_sum(
        &self,
        duel: &mut Duel,
        player: u8,
        value: impl Fn(CardId, &Duel) -> u32,
        acc: u32,
        min: i32,
        max: i32,
        check: impl Fn(&[u32], usize) -> bool,
    ) -> Script<CardSet> {
        if player > 1 {
            return Ok(CardSet::default());
        }
        let must = duel.core.must_select_cards.get();
        let select = self.difference(must);
        duel.core.select_cards = select.iter().collect();
        let params: Vec<(CardId, u32)> = must.iter().chain(select.iter()).map(|card| (card, value(card, duel))).collect();
        let values: List<u32, 16> = params.iter().map(|&(_, param)| param).collect();
        if !check(&values, must.len()) {
            duel.core.must_select_cards.take();
            return Ok(CardSet::default());
        }
        duel.emplace_process(SelectSum {
            playerid: player,
            acc: acc as i32,
            min,
            max,
            params,
            ..SelectSum::default()
        });
        duel.yield_script()?;
        duel.core.must_select_cards.take();
        Ok(duel.return_cards.list.iter().copied().collect())
    }
}

// field::check_with_sum_limit_m: the first `must_count` values are taken, then check_with_sum_limit for the others.
fn check_with_sum_limit_m(values: &[u32], acc: i32, index: usize, min: usize, max: usize, must_count: usize) -> bool {
    if acc == 0 {
        return index == must_count && min == 0;
    }
    if index == must_count {
        return check_with_sum_limit(values, acc, index, 1, min, max);
    }
    let Some(&value) = values.get(index) else {
        return false;
    };
    let (op1, op2) = sum_halves(value);
    (acc >= op1 && check_with_sum_limit_m(values, acc - op1, index + 1, min, max, must_count))
        || (op2 != 0 && acc >= op2 && check_with_sum_limit_m(values, acc - op2, index + 1, min, max, must_count))
}

fn check_with_sum_greater_limit_m(values: &[u32], acc: i32, index: usize, opmin: i32, must_count: usize) -> bool {
    if acc <= 0 {
        return index == must_count && acc + opmin > 0;
    }
    if index == must_count {
        return check_with_sum_greater_limit(values, acc, index, opmin);
    }
    let Some(&value) = values.get(index) else {
        return false;
    };
    let (op1, op2) = sum_halves(value);
    check_with_sum_greater_limit_m(values, acc - op1, index + 1, opmin.min(op1), must_count)
        || (op2 != 0 && check_with_sum_greater_limit_m(values, acc - op2, index + 1, opmin.min(op2), must_count))
}

fn check_with_sum_greater_limit(values: &[u32], acc: i32, index: usize, opmin: i32) -> bool {
    let mut index = index;
    while let Some(&value) = values.get(index) {
        let (op1, op2) = sum_halves(value);
        if (acc <= op1 && acc + opmin > op1) || (op2 != 0 && acc <= op2 && acc + opmin > op2) {
            return true;
        }
        index += 1;
        if check_with_sum_greater_limit(values, acc - op1, index, opmin.min(op1)) {
            return true;
        }
        if op2 != 0 && check_with_sum_greater_limit(values, acc - op2, index, opmin.min(op2)) {
            return true;
        }
    }
    false
}

// field::check_with_sum_limit.
fn check_with_sum_limit(values: &[u32], acc: i32, index: usize, count: usize, min: usize, max: usize) -> bool {
    if count > max {
        return false;
    }
    let mut index = index;
    while let Some(&value) = values.get(index) {
        let (op1, op2) = sum_halves(value);
        if (op1 == acc || op2 == acc) && count >= min {
            return true;
        }
        index += 1;
        if acc > op1 && check_with_sum_limit(values, acc - op1, index, count + 1, min, max) {
            return true;
        }
        if op2 != 0 && acc > op2 && check_with_sum_limit(values, acc - op2, index, count + 1, min, max) {
            return true;
        }
    }
    false
}
