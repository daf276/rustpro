// Port of libgroup.cpp: groups are CardSet values, so Group.CreateGroup is CardSet::default() and `#g` is len().
use crate::card::{CardId, CardSet};
use crate::constants::*;
use crate::duel::Duel;
use crate::processor::SelectUnselectCard;
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

    // Group.CheckWithSumEqual: whether `min` to `max` of the cards add up to `acc`, each counting the low or the high
    // half of its value. Nothing sets the cards a selection must include, so every card is optional.
    pub fn check_with_sum_equal(&self, duel: &Duel, value: impl Fn(CardId, &Duel) -> u32, acc: u32, min: usize, max: usize) -> bool {
        let max = max.max(min);
        if acc == 0 {
            return min == 0;
        }
        let values: crate::list::List<u32, 16> = self.iter().map(|card| value(card, duel)).collect();
        check_with_sum_limit(&values, acc as i32, 0, 1, min, max)
    }
}

// field::check_with_sum_limit.
fn check_with_sum_limit(values: &[u32], acc: i32, index: usize, count: usize, min: usize, max: usize) -> bool {
    if count > max {
        return false;
    }
    let mut index = index;
    while let Some(&value) = values.get(index) {
        let (op1, op2) = ((value & 0xffff) as i32, ((value >> 16) & 0xffff) as i32);
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
