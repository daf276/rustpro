// Port of libduel.cpp. Functions that yield in the core run their units before they return.
use crate::card::{CardId, CardSet};
use crate::chain::ExecuteKind;
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{CounterFilter, CustomCounter, EffectId, EffectSet, Event, OpTarget, Value};

use crate::field::FromEx;
use crate::processor::{SelectCard, SelectOption, SelectYesNo};
use crate::script::{FusionCheck, Procedure, Script};

// What Duel.GetChainInfo pushes for one CHAININFO flag.
#[derive(Debug, Clone, Copy)]
pub enum ChainInfo {
    Int(i64),
    Effect(EffectId),
    Cards(Option<CardSet>),
}

// The Lua value a script reads a CHAININFO flag as; a missing chain or another kind of value reads as the default.
pub trait FromChainInfo: Default {
    fn from_chain_info(info: ChainInfo) -> Self;
}

macro_rules! int_chain_info {
    ($($type:ty),*) => {$(
        impl FromChainInfo for $type {
            fn from_chain_info(info: ChainInfo) -> Self {
                match info {
                    ChainInfo::Int(value) => value as $type,
                    _ => 0,
                }
            }
        }
    )*};
}

int_chain_info!(u8, u16, u32, i32, i64);

// A value Lua reads as nil without a chain link.
macro_rules! optional_int_chain_info {
    ($($type:ty),*) => {$(
        impl FromChainInfo for Option<$type> {
            fn from_chain_info(info: ChainInfo) -> Self {
                match info {
                    ChainInfo::Int(value) => Some(value as $type),
                    _ => None,
                }
            }
        }
    )*};
}

optional_int_chain_info!(u8, u32);

impl FromChainInfo for Option<EffectId> {
    fn from_chain_info(info: ChainInfo) -> Self {
        match info {
            ChainInfo::Effect(effect) => Some(effect),
            _ => None,
        }
    }
}

impl FromChainInfo for Option<CardSet> {
    fn from_chain_info(info: ChainInfo) -> Self {
        match info {
            ChainInfo::Cards(cards) => cards,
            _ => None,
        }
    }
}

const CHAININFO_TRIGGERING_EFFECT: u32 = 1;
const CHAININFO_TRIGGERING_PLAYER: u32 = 2;
const CHAININFO_TRIGGERING_CONTROLER: u32 = 3;
const CHAININFO_TRIGGERING_LOCATION: u32 = 4;
const CHAININFO_TRIGGERING_LOCATION_SYMBOLIC: u32 = 5;
const CHAININFO_TRIGGERING_SEQUENCE: u32 = 6;
const CHAININFO_TRIGGERING_TYPE: u32 = 19;
const CHAININFO_TARGET_CARDS: u32 = 8;
const CHAININFO_TARGET_PLAYER: u32 = 9;
const CHAININFO_TARGET_PARAM: u32 = 10;
const CHAININFO_DISABLE_REASON: u32 = 11;
const CHAININFO_DISABLE_PLAYER: u32 = 12;
const CHAININFO_CHAIN_ID: u32 = 13;

impl Duel {
    pub fn is_turn_player(&self, player: u8) -> bool {
        self.infos.turn_player == player
    }

    pub fn get_current_phase(&self) -> u32 {
        u32::from(self.infos.phase)
    }

    pub fn get_reason_effect(&self) -> Option<EffectId> {
        self.core.reason_effect.get()
    }

    pub fn get_reason_player(&self) -> u8 {
        self.core.reason_player.get()
    }

    pub fn get_current_chain(&self) -> u8 {
        self.core.current_chain.len() as u8
    }

    pub fn is_chain_solving(&self) -> bool {
        self.core.chain_solving
    }

    pub fn register_effect_core(&mut self, effect: EffectId, player: u8) {
        if player <= 1 {
            self.add_effect(effect, player);
        }
    }

    pub fn is_player_can_draw(&self, player: u8, count: usize) -> bool {
        player <= 1 && self.field_is_player_can_draw(player) && self.player[player].list_main.len() >= count
    }

    pub fn equip(&mut self, player: u8, card: CardId, target: CardId) -> Script<bool> {
        self.equip_full(player, card, target, true, false)
    }

    pub fn equip_full(&mut self, player: u8, card: CardId, target: CardId, up: bool, step: bool) -> Script<bool> {
        if player > 1 {
            return Ok(false);
        }
        self.emplace_process(crate::operations::Equip {
            step: 0,
            equip_player: player,
            equip_card: card,
            target,
            faceup: up,
            is_step: step,
        });
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }

    pub fn get_tribute_group(&self, card: CardId) -> CardSet {
        let mut group = CardSet::default();
        let mut ex_list = CardSet::default();
        self.summon_release_list(card, &mut group, &mut ex_list, None, false, 0xff00ff);
        group.union(ex_list)
    }

    // Duel.CheckTribute with every argument; toplayer is the card's controller where Lua leaves it out.
    pub fn check_tribute(&self, card: CardId, min: i32, max: i32, mg: Option<CardSet>, toplayer: u8, zone: u32) -> bool {
        self.field_check_tribute(card, min, max, mg, toplayer, zone, 0xff00ff)
    }

    pub fn select_tribute(
        &mut self,
        player: u8,
        card: CardId,
        min: i32,
        max: i32,
        mg: Option<CardSet>,
        toplayer: u8,
        zone: u32,
        cancelable: bool,
    ) -> Script<CardSet> {
        if player > 1 || toplayer > 1 {
            return Ok(CardSet::default());
        }
        let (mut release_cards, mut release_cards_ex) = (CardSet::default(), CardSet::default());
        self.get_summon_release_list(card, &mut release_cards, &mut release_cards_ex, mg, toplayer != player, 0xff00ff);
        self.core.release_cards = release_cards;
        self.core.release_cards_ex = release_cards_ex;
        self.core.release_cards_ex_oneof.clear();
        self.select_tribute_cards(None, player, cancelable, min, max, toplayer, zone);
        self.yield_script()?;
        Ok(self.returned_cards())
    }

    pub fn is_player_can_discard_deck(&self, player: u8, count: usize) -> bool {
        player <= 1
            && self.player[player].list_main.len() >= count
            && self.is_player_affected_by_effect(player, EFFECT_CANNOT_DISCARD_DECK).is_none()
    }

    pub fn discard_deck(&mut self, player: u8, count: u16, reason: u32) -> Script<i32> {
        self.emplace_process(crate::operations::DiscardDeck {
            playerid: player,
            count,
            reason,
            ..Default::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn get_location_count(&self, player: u8, location: u32) -> i32 {
        if player > 1 {
            return 0;
        }
        let uplayer = self.core.reason_player.get();
        self.get_useable_count(None, player, location as u8, uplayer, LOCATION_REASON_TOFIELD, 0xff)
            .0
    }

    // The matching cards of `filter`; Lua's extra filter arguments are captures of the closure.
    fn matching_cards(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
        count: usize,
        is_target: bool,
    ) -> (bool, CardSet) {
        let mut group = CardSet::default();
        let exception = exception.unwrap_or_default();
        let found = self.filter_matching_card(
            &mut |duel, card| filter(card, duel),
            player,
            location1,
            location2,
            &mut group,
            &exception,
            count,
            is_target,
        );
        (found, group)
    }

    pub fn is_existing_matching_card(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        count: usize,
        exception: Option<CardSet>,
    ) -> bool {
        self.matching_cards(filter, player, location1, location2, exception, count, false).0
    }

    pub fn is_existing_target(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        count: usize,
        exception: Option<CardSet>,
    ) -> bool {
        self.matching_cards(filter, player, location1, location2, exception, count, true).0
    }

    pub fn get_matching_group(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> CardSet {
        self.matching_cards(filter, player, location1, location2, exception, 0, false).1
    }

    pub fn hint(&mut self, hint_type: u8, player: u8, description: u64) {
        if player > 1 {
            return;
        }
        let player = if hint_type == HINT_OPSELECTED { 1 - player } else { player };
        self.trace.message(MSG_HINT).u8(hint_type).u8(player).u64(description);
    }

    pub(crate) fn select_cards(&mut self, player: u8, cards: CardSet, cancelable: bool, min: u8, max: u8) -> Script<()> {
        self.core.select_cards = cards.iter().collect();
        self.emplace_process(SelectCard {
            playerid: player,
            cancelable,
            min,
            max,
            ..SelectCard::default()
        });
        self.yield_script()
    }

    pub fn select_matching_card(
        &mut self,
        select_player: u8,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        min: u8,
        max: u8,
        exception: Option<CardSet>,
    ) -> Script<CardSet> {
        self.select_matching_card_full(select_player, filter, player, location1, location2, min, max, false, exception)
    }

    pub fn select_matching_card_full(
        &mut self,
        select_player: u8,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        min: u8,
        max: u8,
        cancel: bool,
        exception: Option<CardSet>,
    ) -> Script<CardSet> {
        if select_player > 1 {
            return Ok(CardSet::default());
        }
        let cards = self.matching_cards(filter, player, location1, location2, exception, 0, false).1;
        self.select_cards(select_player, cards, cancel, min, max)?;
        Ok(self.returned_cards())
    }

    pub(crate) fn returned_cards(&self) -> CardSet {
        match self.return_cards.canceled {
            true => CardSet::default(),
            false => self.return_cards.list.iter().copied().collect(),
        }
    }

    pub fn select_target(
        &mut self,
        select_player: u8,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        min: u8,
        max: u8,
        exception: Option<CardSet>,
    ) -> Script<Option<CardSet>> {
        if select_player > 1 || self.core.current_chain.is_empty() {
            return Ok(None);
        }
        let cards = self.matching_cards(filter, player, location1, location2, exception, 0, true).1;
        self.select_cards(select_player, cards, false, min, max)?;
        if self.return_cards.canceled {
            return Ok(None);
        }
        let selected = self.returned_cards();
        let Some(chain) = self.get_chain(0) else { return Ok(None) };
        let targets = chain.target_cards.unwrap_or_default().union(selected);
        chain.target_cards = Some(targets);
        let (effect, chain_id) = (chain.triggering_effect, chain.chain_id);
        if self.effects[effect].effect_type & EFFECT_TYPE_CONTINUOUS != 0 {
            return Ok(Some(targets));
        }
        for card in selected {
            self.create_relation(card, effect, chain_id);
            if self.effects[effect].is_flag(EFFECT_FLAG_CARD_TARGET) {
                let location = self.cards.get_info_location(card);
                self.trace.message(MSG_BECOME_TARGET).u32(1).loc(location);
            }
        }
        Ok(Some(selected))
    }

    pub fn get_first_target(&self) -> Option<CardId> {
        self.get_chain_ref(0)?.target_cards?.get_first()
    }

    // A card is taken as a group of one, which replaces rather than adds to a continuous effect's targets.
    pub fn set_target_card(&mut self, targets: impl Into<CardSet>) {
        let targets = targets.into();
        let Some(chain) = self.get_chain_ref(0) else { return };
        let (effect, chain_id) = (chain.triggering_effect, chain.chain_id);
        let continuous = self.effects[effect].effect_type & EFFECT_TYPE_CONTINUOUS != 0;
        let Some(chain) = self.get_chain(0) else { return };
        if continuous {
            chain.target_cards = Some(targets);
            return;
        }
        chain.target_cards = Some(chain.target_cards.unwrap_or_default().union(targets));
        for card in targets {
            self.create_relation(card, effect, chain_id);
        }
        if self.effects[effect].is_flag(EFFECT_FLAG_CARD_TARGET) {
            let locations: Vec<_> = targets.iter().map(|card| self.cards.get_info_location(card)).collect();
            let mut message = self.trace.message(MSG_BECOME_TARGET);
            message.u32(locations.len() as u32);
            for location in locations {
                message.loc(location);
            }
        }
    }

    pub fn set_target_player(&mut self, player: u8) {
        if let Some(chain) = self.get_chain(0) {
            chain.target_player = player;
        }
    }

    pub fn set_target_param(&mut self, param: u32) {
        if let Some(chain) = self.get_chain(0) {
            chain.target_param = param as i32;
        }
    }

    pub fn set_operation_info(&mut self, chain_count: u8, category: u64, targets: Option<CardSet>, count: u8, player: u8, param: i32) {
        if let Some(chain) = self.get_chain(chain_count) {
            chain.opinfos.set(
                category,
                OpTarget {
                    op_cards: targets,
                    op_count: count,
                    op_player: player,
                    op_param: param,
                },
            );
        }
    }

    pub fn get_operation_info(&self, chain_count: u8, category: u64) -> Option<OpTarget> {
        self.get_chain_ref(chain_count)?.opinfos.get(category).copied()
    }

    pub fn get_possible_operation_info(&self, chain_count: u8, category: u64) -> Option<OpTarget> {
        self.get_chain_ref(chain_count)?.possibleopinfos.get(category).copied()
    }

    pub fn get_chain_info<T: FromChainInfo>(&self, chain_count: u8, info: u32) -> T {
        let Some(chain) = self.get_chain_ref(chain_count) else {
            return T::default();
        };
        let int = |value: i64| ChainInfo::Int(value);
        T::from_chain_info(match info {
            CHAININFO_TRIGGERING_EFFECT => ChainInfo::Effect(chain.triggering_effect),
            CHAININFO_TRIGGERING_PLAYER => int(chain.triggering_player.into()),
            CHAININFO_TRIGGERING_CONTROLER => int(chain.triggering_controler.into()),
            CHAININFO_TRIGGERING_LOCATION => {
                int((chain.triggering_location & !(LOCATION_STZONE | LOCATION_MMZONE | LOCATION_EMZONE)).into())
            }
            CHAININFO_TRIGGERING_LOCATION_SYMBOLIC => int((chain.triggering_location & !u16::from(LOCATION_MZONE | LOCATION_SZONE)).into()),
            CHAININFO_TRIGGERING_SEQUENCE => int(chain.triggering_sequence.into()),
            CHAININFO_TRIGGERING_TYPE => int(chain.triggering_state.card_type.into()),
            CHAININFO_TARGET_CARDS => ChainInfo::Cards(chain.target_cards),
            CHAININFO_TARGET_PLAYER => int(chain.target_player.into()),
            CHAININFO_TARGET_PARAM => int(chain.target_param.into()),
            CHAININFO_DISABLE_REASON => chain.disable_reason.map_or(ChainInfo::Cards(None), ChainInfo::Effect),
            CHAININFO_DISABLE_PLAYER => int(chain.disable_player.into()),
            CHAININFO_CHAIN_ID => int(chain.chain_id.into()),
            // The other flags come with the scripts that read them.
            _ => {
                debug_assert!(false, "unported CHAININFO flag {info}");
                ChainInfo::Cards(None)
            }
        })
    }

    pub fn is_chain_disablable(&self, chain_count: u8) -> bool {
        !self.core.chain_solving || self.field_is_chain_disablable(chain_count)
    }

    pub fn negate_effect(&mut self, chain_count: u8) -> bool {
        self.disable_chain(chain_count)
    }

    pub fn destroy(&mut self, targets: impl Into<CardSet>, reason: u32) -> Script<i32> {
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.field_destroy(targets.into(), reason_effect, reason, reason_player, PLAYER_NONE, LOCATION_GRAVE, 0);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn send_to_grave(&mut self, targets: impl Into<CardSet>, reason: u32) -> Script<i32> {
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            PLAYER_NONE,
            LOCATION_GRAVE,
            0,
            POS_FACEUP as u8,
            false,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn send_to_hand(&mut self, targets: impl Into<CardSet>, player: u8, reason: u32) -> Script<i32> {
        if player > PLAYER_NONE {
            return Ok(0);
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            player,
            LOCATION_HAND,
            0,
            POS_FACEUP as u8,
            false,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn special_summon(
        &mut self,
        targets: impl Into<CardSet>,
        sumtype: u32,
        sumplayer: u8,
        target_player: u8,
        nocheck: bool,
        nolimit: bool,
        positions: u32,
    ) -> Script<i32> {
        if sumplayer >= PLAYER_NONE {
            return Ok(0);
        }
        self.field_special_summon(
            targets.into(),
            sumtype,
            sumplayer,
            target_player,
            nocheck,
            nolimit,
            positions as u8,
            0xff,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn draw(&mut self, player: u8, count: u32, reason: u32) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.field_draw(reason_effect, reason, reason_player, player, count as u16);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn damage(&mut self, player: u8, amount: u32, reason: u32) -> Script<u32> {
        if player > 1 {
            return Ok(0);
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.field_damage(reason_effect, reason, reason_player, None, player, amount);
        self.yield_script()?;
        Ok(self.returns.at_u32(0))
    }

    pub fn recover(&mut self, player: u8, amount: u32, reason: u32) -> Script<u32> {
        if player > 1 {
            return Ok(0);
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.field_recover(reason_effect, reason, reason_player, player, amount);
        self.yield_script()?;
        Ok(self.returns.at_u32(0))
    }

    pub fn confirm_cards(&mut self, player: u8, targets: impl Into<CardSet>) -> Script<()> {
        let targets = targets.into();
        if player > 1 || targets.is_empty() {
            return Ok(());
        }
        let locations: Vec<_> = targets
            .iter()
            .map(|card| (self.cards[card].data.code, self.cards[card].current))
            .collect();
        {
            let mut message = self.trace.message(MSG_CONFIRM_CARDS);
            message.u8(player).u32(locations.len() as u32);
            for (code, current) in locations {
                message.u32(code).u8(current.controler).u8(current.location).u32(current.sequence);
            }
        }
        let reason = if self.core.chain_solving { REASON_EFFECT } else { REASON_COST };
        let (reason_effect, reason_player, revealing_player) = (self.core.reason_effect.get(), self.core.reason_player.get(), 1 - player);
        let during_to_hand = self.check_event(EVENT_TO_HAND).is_some();
        let mut hand = CardSet::default();
        for card in targets {
            self.raise_single_event(card, EVENT_CONFIRM, reason_effect, reason, reason_player, revealing_player, 0);
            if during_to_hand && self.cards[card].current.location == LOCATION_HAND {
                self.raise_single_event(
                    card,
                    EVENT_TOHAND_CONFIRM,
                    reason_effect,
                    reason,
                    reason_player,
                    revealing_player,
                    0,
                );
                hand.insert(card);
            }
        }
        self.raise_event(
            Some(targets),
            EVENT_CONFIRM,
            reason_effect,
            reason,
            reason_player,
            revealing_player,
            0,
        );
        if !hand.is_empty() {
            self.raise_event(
                Some(hand),
                EVENT_TOHAND_CONFIRM,
                reason_effect,
                reason,
                reason_player,
                revealing_player,
                0,
            );
        }
        self.process_single_event();
        self.process_instant_event();
        self.yield_script()
    }

    pub fn shuffle_hand(&mut self, player: u8) -> Script<()> {
        if player > 1 {
            return Ok(());
        }
        self.shuffle(player, LOCATION_HAND);
        self.yield_script()
    }

    // The values a script keeps in Lua locals across calls, per duel; 0 until set.
    pub fn get_script_state(&self, key: u64) -> i64 {
        self.script_state
            .iter()
            .find(|(other, _)| *other == key)
            .map_or(0, |&(_, value)| value)
    }

    pub fn set_script_state(&mut self, key: u64, value: i64) {
        match self.script_state.iter_mut().find(|(other, _)| *other == key) {
            Some((_, slot)) => *slot = value,
            None => self.script_state.push((key, value)),
        }
    }
}

// What Duel.GetLocationCountFromEx's fourth parameter names: the monster to summon, or a type the core gives its
// temporary card in the Extra Deck.
#[derive(Debug, Clone, Copy)]
pub enum ExtraSummon {
    Card(CardId),
    Type(u32),
}

impl From<CardId> for ExtraSummon {
    fn from(card: CardId) -> Self {
        ExtraSummon::Card(card)
    }
}

impl From<u32> for ExtraSummon {
    fn from(card_type: u32) -> Self {
        ExtraSummon::Type(card_type)
    }
}

impl Duel {
    pub fn get_turn_player(&self) -> u8 {
        self.infos.turn_player
    }

    pub fn is_main_phase(&self) -> bool {
        let phase = self.infos.phase;
        phase == PHASE_MAIN1 || phase == PHASE_MAIN2
    }

    pub fn is_phase(&self, phase: u32) -> bool {
        u32::from(self.infos.phase) == phase
    }

    pub fn remove(&mut self, targets: impl Into<CardSet>, position: u32, reason: u32) -> Script<i32> {
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            PLAYER_NONE,
            LOCATION_REMOVED,
            0,
            position as u8,
            false,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    // Sequence -2 sends to no location; SEQ_DECKSHUFFLE places on top and shuffles at the next check.
    pub fn send_to_deck(&mut self, targets: impl Into<CardSet>, player: u8, sequence: i32, reason: u32) -> Script<i32> {
        if player > PLAYER_NONE {
            return Ok(0);
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        let location = if sequence == -2 { 0 } else { LOCATION_DECK };
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            player,
            location,
            sequence as u8,
            POS_FACEUP as u8,
            false,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn move_to_field_core(
        &mut self,
        card: CardId,
        move_player: u8,
        player: u8,
        destination: u32,
        positions: u32,
        enable: bool,
    ) -> Script<bool> {
        if player > 1 {
            return Ok(false);
        }
        self.enable_field_effect(card, false);
        self.adjust_instant();
        self.field_move_to_field(
            card,
            move_player,
            player,
            destination as u16,
            positions as u8,
            enable,
            0xff,
            false,
            LOCATION_REASON_NONE,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }

    pub fn swap_sequence_core(&mut self, card1: CardId, card2: CardId) {
        let (player, location) = (self.cards[card1].current.controler, self.cards[card1].current.location);
        let reason_effect = self.core.reason_effect.get();
        if self.cards[card2].current.controler != player
            || (location != LOCATION_MZONE && location != LOCATION_SZONE)
            || self.cards[card2].current.location != location
            || !self.is_affect_by_effect(card1, reason_effect)
            || !self.is_affect_by_effect(card2, reason_effect)
        {
            return;
        }
        self.swap_card(card1, card2);
        let reason_player = self.core.reason_player.get();
        for card in [card1, card2] {
            self.raise_single_event(card, EVENT_MOVE, reason_effect, 0, reason_player, player, 0);
        }
        self.raise_event(
            Some(CardSet::from_iter([card1, card2])),
            EVENT_MOVE,
            reason_effect,
            0,
            reason_player,
            player,
            0,
        );
        self.process_single_event();
        self.process_instant_event();
    }

    pub fn get_field_card(&self, player: u8, location: u32, sequence: u32) -> Option<CardId> {
        if player > 1 {
            return None;
        }
        self.field_card(player, location as u16, sequence)
            .filter(|&card| !self.cards[card].get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP))
    }

    pub fn get_field_group(&self, player: u8, location1: u32, location2: u32) -> CardSet {
        self.filter_field_card(player, location1, location2)
    }

    pub fn get_field_group_count(&self, player: u8, location1: u32, location2: u32) -> usize {
        self.filter_field_card(player, location1, location2).len()
    }

    pub fn get_decktop_group(&self, player: u8, count: usize) -> CardSet {
        let deck = &self.player[player].list_main;
        deck.iter().skip(deck.len().saturating_sub(count)).copied().collect()
    }

    pub fn get_operated_group(&self) -> CardSet {
        self.core.operated_set
    }

    pub fn break_effect(&mut self) -> Script<()> {
        self.field_break_effect(true);
        self.raise_event(None, EVENT_BREAK_EFFECT, None, 0, PLAYER_NONE, PLAYER_NONE, 0);
        self.process_instant_event();
        self.yield_script()
    }

    pub fn adjust_instantly(&mut self, card: Option<CardId>) {
        if let Some(card) = card {
            self.filter_disable_related_cards(card);
        }
        self.adjust_instant();
    }

    pub fn disable_shuffle_check(&mut self, disable: bool) {
        self.core.shuffle_check_disabled = disable;
    }

    pub fn shuffle_extra(&mut self, player: u8) -> Script<()> {
        if player > 1 {
            return Ok(());
        }
        self.shuffle(player, LOCATION_EXTRA);
        self.yield_script()
    }

    pub fn discard_hand(&mut self, player: u8, filter: impl Fn(CardId, &Duel) -> bool, min: u8, max: u8, reason: u32) -> Script<i32> {
        self.discard_hand_full(player, filter, min, max, reason, None)
    }

    // The DiscardHand unit in one: the selection, then the cards selected to the GY.
    pub fn discard_hand_full(
        &mut self,
        player: u8,
        filter: impl Fn(CardId, &Duel) -> bool,
        min: u8,
        max: u8,
        reason: u32,
        exception: Option<CardSet>,
    ) -> Script<i32> {
        let cards = self
            .matching_cards(filter, player, u32::from(LOCATION_HAND), 0, exception, 0, false)
            .1;
        if cards.is_empty() {
            return Ok(0);
        }
        let description = if reason & REASON_DISCARD != 0 { 501 } else { 504 };
        self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(player).u64(description);
        self.select_cards(player, cards, false, min, max)?;
        if self.return_cards.list.is_empty() {
            return Ok(0);
        }
        let selected = self.return_cards.list.iter().copied().collect();
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.send_to(
            selected,
            reason_effect,
            reason,
            reason_player,
            PLAYER_NONE,
            LOCATION_GRAVE,
            0,
            POS_FACEUP as u8,
            false,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn select_yes_no(&mut self, player: u8, description: u64) -> Script<bool> {
        if player > 1 {
            return Ok(false);
        }
        self.emplace_process(SelectYesNo {
            playerid: player,
            description,
            ..SelectYesNo::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }

    // The index of the chosen option, announced to the opponent (the default sel_hint).
    pub fn select_option(&mut self, player: u8, options: &[u64]) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        self.core.select_options = options.iter().copied().collect();
        self.emplace_process(SelectOption {
            playerid: player,
            ..SelectOption::default()
        });
        self.yield_script()?;
        let selected = self.returns.at_i32(0);
        if let Some(&option) = usize::try_from(selected).ok().and_then(|index| self.core.select_options.get(index)) {
            self.trace.message(MSG_HINT).u8(HINT_OPSELECTED).u8(player).u64(option);
        }
        Ok(selected)
    }

    pub fn hint_selection(&mut self, targets: impl Into<CardSet>) {
        let targets = targets.into();
        let locations: Vec<_> = targets.iter().map(|card| self.cards.get_info_location(card)).collect();
        let mut message = self.trace.message(MSG_CARD_SELECTED);
        message.u32(locations.len() as u32);
        for location in locations {
            message.loc(location);
        }
    }

    pub fn negate_activation(&mut self, chain_count: u8) -> bool {
        self.negate_chain(chain_count)
    }

    // Duel.IsChainNegatable answers true for every chain link.
    pub fn is_chain_negatable(&self, _chain_count: u8) -> bool {
        true
    }

    pub fn negate_related_chain(&mut self, card: CardId, reset_flag: u32) {
        let Some(reason_effect) = self.core.reason_effect.get() else {
            return;
        };
        if self.core.current_chain.len() < 2 || !self.is_affect_by_effect(card, Some(reason_effect)) {
            return;
        }
        let chains: Vec<(EffectId, u16)> = self
            .core
            .current_chain
            .iter()
            .rev()
            .map(|chain| (chain.triggering_effect, chain.chain_id))
            .collect();
        for (effect, chain_id) in chains {
            if self.get_handler(effect) != card || !self.is_has_relation(card, effect, chain_id) {
                continue;
            }
            let negate = self.new_effect(self.get_handler(reason_effect));
            let peffect = &mut self.effects[negate];
            peffect.effect_type = EFFECT_TYPE_SINGLE;
            peffect.code = EFFECT_DISABLE_CHAIN;
            peffect.value = Value::Int(i32::from(chain_id));
            peffect.reset_flag = RESET_CHAIN | RESET_EVENT | reset_flag;
            self.card_add_effect(card, negate);
        }
    }

    pub fn get_mzone_count(&self, player: u8, excluded: Option<CardSet>) -> i32 {
        self.get_mzone_count_full(player, excluded, self.core.reason_player.get(), LOCATION_REASON_TOFIELD, 0xff)
    }

    pub fn get_mzone_count_full(&self, player: u8, excluded: Option<CardSet>, uplayer: u8, reason: u32, zone: u32) -> i32 {
        if player > 1 {
            return 0;
        }
        let zones = excluded.map_or_else(|| self.zones(), |excluded| self.zones_without(excluded));
        self.get_useable_count_other_in(&zones, None, player, LOCATION_MZONE, uplayer, reason, zone)
            .0
    }

    pub fn is_duel_type(&self, flag: u64) -> bool {
        self.is_flag(flag)
    }

    pub fn get_location_count_from_ex(&self, player: u8, uplayer: u8, excluded: Option<CardSet>, summoned: impl Into<ExtraSummon>) -> i32 {
        if player > 1 {
            return 0;
        }
        let zones = excluded.map_or_else(|| self.zones(), |excluded| self.zones_without(excluded));
        let summoned = match summoned.into() {
            ExtraSummon::Card(card) => FromEx::card(self, Some(card)),
            ExtraSummon::Type(card_type) => FromEx::card_type(card_type),
        };
        self.get_useable_count_fromex_in(&zones, summoned, player, uplayer, 0xff).0
    }

    pub fn get_usable_mzone_count(&self, player: u8) -> i32 {
        if player > 1 {
            return 0;
        }
        let uplayer = self.core.reason_player.get();
        let (count1, flag1) = self.get_tofield_count(None, player, LOCATION_MZONE, uplayer, LOCATION_REASON_TOFIELD, 0xff);
        let (count2, flag2) = self.get_spsummonable_count_fromex(None, player, uplayer, 0xff);
        let both = (!(flag1 | flag2) & 0x1f).count_ones() as i32;
        (count1 + count2 - both).min(self.get_mzone_limit(player, uplayer, LOCATION_REASON_TOFIELD))
    }

    pub fn register_flag_effect(&mut self, player: u8, code: u32, reset: u32, flag: u32, count: u16) -> Option<EffectId> {
        if player > 1 {
            return None;
        }
        let mut reset = reset;
        if reset & RESET_PHASE != 0 && reset & (RESET_SELF_TURN | RESET_OPPO_TURN) == 0 {
            reset |= RESET_SELF_TURN | RESET_OPPO_TURN;
        }
        let effect = self.new_effect(CardId::default());
        let peffect = &mut self.effects[effect];
        peffect.effect_owner = player;
        peffect.effect_type = EFFECT_TYPE_FIELD;
        peffect.code = (code & 0xfff_ffff) | 0x1000_0000;
        peffect.reset_flag = reset;
        peffect.flag[0] = flag | EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_PLAYER_TARGET | EFFECT_FLAG_FIELD_ONLY;
        peffect.s_range = 1;
        peffect.o_range = 0;
        peffect.reset_count = count.max(1);
        peffect.label = crate::effect::Label::new(&[0]);
        self.add_effect(effect, player);
        Some(effect)
    }

    pub fn get_flag_effect(&self, player: u8, code: u32) -> usize {
        if player > 1 {
            return 0;
        }
        self.filter_player_effect(player, (code & 0xfff_ffff) | 0x1000_0000, true).len()
    }

    // Not in the Lua API: false when no effect of this code is registered anywhere (true may be a false alarm), for
    // library shortcuts that hold only without such effects.
    pub fn may_have_effect(&self, code: u32) -> bool {
        self.field_effects.registry.has_code(code)
    }

    pub fn get_player_effect(&self, player: u8, code: u32) -> crate::effect::EffectSet {
        if player > 1 {
            return crate::effect::EffectSet::default();
        }
        self.filter_player_effect(player, code, true)
    }

    // A group a script keeps in Lua across calls (a group label object or chain data), per duel.
    pub fn get_script_group(&self, key: u64) -> Option<CardSet> {
        self.script_groups.iter().find(|(other, _)| *other == key).map(|&(_, group)| group)
    }

    pub fn set_script_group(&mut self, key: u64, group: Option<CardSet>) {
        self.script_groups.retain(|(other, _)| *other != key);
        if let Some(group) = group {
            self.script_groups.push((key, group));
        }
    }

    pub fn is_summon_cancelable(&self) -> bool {
        self.core.summon_cancelable
    }

    // A second counter with the same id is ignored.
    // Only chain counters are counted; the other activities come with the cards that count them.
    pub fn add_custom_activity_counter(&mut self, id: u32, activity: u8, filter: CounterFilter) {
        if activity != ACTIVITY_CHAIN {
            self.reach_unported(concat!(file!(), ":", line!()));
        }
        let counters = &mut self.core.custom_counters;
        if counters.iter().any(|counter| counter.id == id && counter.activity == activity) {
            return;
        }
        counters.push(CustomCounter {
            id,
            activity,
            filter,
            amount: [0; 2],
        });
    }

    pub fn get_custom_activity_count(&self, id: u32, player: u8, activity: u8) -> u32 {
        self.core
            .custom_counters
            .iter()
            .find(|counter| counter.id == id && counter.activity == activity)
            .map_or(0, |counter| counter.amount.get(usize::from(player)).copied().unwrap_or(0))
    }

    pub fn set_possible_operation_info(
        &mut self,
        chain_count: u8,
        category: u64,
        targets: Option<CardSet>,
        count: u8,
        player: u8,
        param: i32,
    ) {
        if let Some(chain) = self.get_chain(chain_count) {
            chain.possibleopinfos.set(
                category,
                OpTarget {
                    op_cards: targets,
                    op_count: count,
                    op_player: player,
                    op_param: param,
                },
            );
        }
    }

    // The player's monster in battle first, then the opponent's.
    pub fn get_battle_monster(&self, player: u8) -> (Option<CardId>, Option<CardId>) {
        let monster = |player: u8| {
            [self.core.attacker, self.core.attack_target]
                .into_iter()
                .flatten()
                .find(|&card| self.cards[card].current.controler == player)
        };
        (monster(player), monster(1 - player))
    }

    pub fn synchro_summon(&mut self, player: u8, card: CardId, must: Option<CardSet>, materials: Option<CardSet>) -> Script<()> {
        self.synchro_summon_full(player, card, must, materials, 0, 0)
    }

    pub fn synchro_summon_full(
        &mut self,
        player: u8,
        card: CardId,
        must: Option<CardSet>,
        materials: Option<CardSet>,
        min: u16,
        max: u16,
    ) -> Script<()> {
        self.spsummon_rule(SUMMON_TYPE_SYNCHRO, player, card, must, materials, min, max)
    }

    pub fn link_summon(&mut self, player: u8, card: CardId, must: Option<CardSet>, materials: Option<CardSet>) -> Script<()> {
        self.link_summon_full(player, card, must, materials, 0, 0)
    }

    pub fn link_summon_full(
        &mut self,
        player: u8,
        card: CardId,
        must: Option<CardSet>,
        materials: Option<CardSet>,
        min: u16,
        max: u16,
    ) -> Script<()> {
        self.spsummon_rule(SUMMON_TYPE_LINK, player, card, must, materials, min, max)
    }

    // A summon by the card's procedures of `summon_type`; during a chain it waits until the chain link has resolved.
    fn spsummon_rule(
        &mut self,
        summon_type: u32,
        player: u8,
        card: CardId,
        must: Option<CardSet>,
        materials: Option<CardSet>,
        min: u16,
        max: u16,
    ) -> Script<()> {
        if player > 1 {
            return Ok(());
        }
        self.core.summon_cancelable = false;
        let max = if min != 0 { max } else { 0 };
        let procedure = Procedure {
            card,
            must,
            materials,
            min,
            max,
            ..Procedure::default()
        };
        self.special_summon_rule(player, card, summon_type, Some(procedure));
        if !self.core.current_chain.is_empty() {
            self.core.reserved = self.core.subunits.pop().map(Box::new);
            self.core.summoning_card = Some(card);
        }
        self.yield_script()
    }

    // spsummonable_rule of libcard.cpp: Card.IsSynchroSummonable and its kin, with the materials passed on to the
    // procedure's condition rather than through core.must_use_mats.
    pub(crate) fn spsummonable_rule(
        &self,
        card: CardId,
        card_type: u32,
        summon_type: u32,
        must: Option<CardSet>,
        materials: Option<CardSet>,
        min: u16,
        max: u16,
    ) -> bool {
        if self.cards[card].data.card_type & card_type == 0 {
            return false;
        }
        let procedure = Procedure {
            card,
            must,
            materials,
            min,
            max: if min != 0 { max } else { 0 },
            ..Procedure::default()
        };

        self.is_special_summonable(card, self.core.reason_player.get(), summon_type, procedure)
    }
}

impl Duel {
    pub fn get_turn_count(&self) -> i32 {
        self.get_turn_count_full(None)
    }

    // `player` None counts the turns of both players.
    pub fn get_turn_count_full(&self, player: Option<u8>) -> i32 {
        match player {
            Some(player) if player > 1 => 0,
            Some(player) => i32::from(self.infos.turn_id_by_player[player]),
            None => i32::from(self.infos.turn_id),
        }
    }

    pub fn release(&mut self, targets: impl Into<CardSet>, reason: u32) -> Script<i32> {
        self.release_full(targets, reason, None)
    }

    // `reason_player` None is the reason player.
    pub fn release_full(&mut self, targets: impl Into<CardSet>, reason: u32, reason_player: Option<u8>) -> Script<i32> {
        let reason_player = reason_player.unwrap_or_else(|| self.core.reason_player.get());
        let reason_effect = self.core.reason_effect.get();
        self.field_release(targets.into(), reason_effect, reason, reason_player);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn get_matching_group_count(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> usize {
        self.get_matching_group(filter, player, location1, location2, exception).len()
    }

    pub fn is_can_remove_counter(&self, player: u8, self_side: u8, oppo: u8, counter_type: u16, count: u16, _reason: u32) -> bool {
        player <= 1 && self.is_player_can_remove_counter(player, None, self_side, oppo, counter_type, count)
    }

    pub fn remove_counter(&mut self, player: u8, self_side: u8, oppo: u8, counter_type: u16, count: u16, reason: u32) -> Script<bool> {
        if player > 1 {
            return Ok(false);
        }
        self.field_remove_counter(reason, None, player, self_side, oppo, counter_type, count);
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }

    // Nothing is assumed until a card calls Card.AssumeProperty.
    pub fn assume_reset(&self) {}
}

impl Duel {
    pub fn return_to_field(&mut self, card: CardId) -> Script<bool> {
        self.return_to_field_full(card, None, 0xff)
    }

    // `position` None is the card's previous position. The core first refreshes the disabled zones, which stay as they
    // are until a card disables zones.
    pub fn return_to_field_full(&mut self, card: CardId, position: Option<u32>, zone: u32) -> Script<bool> {
        let previous = self.cards[card].previous;
        if self.cards[card].current.reason & REASON_TEMPORARY == 0 {
            return Ok(false);
        }
        let position = position.unwrap_or(previous.position);
        self.enable_field_effect(card, false);
        self.adjust_instant();
        unported!(self, self.filter_field_effect(EFFECT_DISABLE_FIELD));
        let (player, location) = (previous.controler, u16::from(previous.location));
        let unit = crate::operations::MoveToField {
            step: 0,
            enable: true,
            pzone: false,
            zone: zone as u8,
            rule: false,
            location_reason: LOCATION_REASON_RETURN_TOFIELD,
            confirm: true,
            ret: 1,
            target: card,
        };
        self.move_to_field_unit(player, player, location, position as u8, unit);
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }
}

impl Duel {
    pub fn sset(&mut self, player: u8, targets: impl Into<CardSet>) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        let targets = targets.into();
        if targets.is_empty() {
            return Ok(self.returns.at_i32(0));
        }
        let reason_effect = self.core.reason_effect.get();
        self.sset_group(player, player, targets, true, reason_effect);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }
}

impl Duel {
    pub fn overlay(&mut self, target: CardId, materials: impl Into<CardSet>) -> Script<()> {
        self.overlay_full(target, materials, false)
    }

    // The Lua errors of attaching to a material or a card to itself stop the oracle's script; no port does either.
    pub fn overlay_full(&mut self, target: CardId, materials: impl Into<CardSet>, send_materials_to_grave: bool) -> Script<()> {
        let materials = materials.into();
        if self.cards[target].overlay_target.is_some() || materials.contains(target) {
            self.reach_unported(concat!(file!(), ":", line!()));
        }
        self.xyz_overlay(target, materials, send_materials_to_grave);
        self.yield_script()
    }

    pub fn get_overlay_group(&self, player: u8, self_side: u8, oppo: u8) -> CardSet {
        if player > 1 {
            return CardSet::default();
        }
        self.field_overlay_group(player, self_side, oppo, None)
    }

    pub fn get_overlay_count(&self, player: u8, self_side: u8, oppo: u8) -> usize {
        self.get_overlay_group(player, self_side, oppo).len()
    }

    pub fn check_remove_overlay_card(&self, player: u8, self_side: u8, oppo: u8, count: u16, _reason: u32) -> bool {
        player <= 1 && self.is_player_can_remove_overlay_card(player, None, self_side, oppo, count)
    }

    pub fn remove_overlay_card(&mut self, player: u8, self_side: u8, oppo: u8, min: u16, max: u16, reason: u32) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        self.field_remove_overlay_card(reason, None, player, self_side, oppo, min, max);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }
}

impl Duel {
    // field::get_fusion_material; cards with EFFECT_EXTRA_FUSION_MATERIAL come with the cards that register it.
    pub fn get_fusion_material_core(&self, player: u8) -> CardSet {
        if player > 1 {
            return CardSet::default();
        }
        let info = &self.player[player];

        let extra = |card: CardId| {
            let extra = self.is_affected_by_effect(card, EFFECT_EXTRA_FUSION_MATERIAL);
            unported!(self, extra);
            extra.is_some()
        };
        let mut material: CardSet = info.list_mzone.iter().flatten().copied().collect();
        material.merge(info.list_szone.iter().flatten().copied().filter(|&card| extra(card)).collect());
        let hand = info.list_hand.iter().copied();
        material.merge(
            hand.filter(|&card| self.cards[card].data.card_type & TYPE_MONSTER != 0 || extra(card))
                .collect(),
        );
        material.merge(info.list_grave.iter().copied().filter(|&card| extra(card)).collect());
        material
    }

    // field::get_ritual_material for the reason effect. Monsters in the GY, Deck or Extra Deck and Xyz materials become
    // materials through EFFECT_EXTRA_RITUAL_MATERIAL, which comes with the cards that register it.
    pub fn get_ritual_material(&self, player: u8, check_level: bool) -> CardSet {
        if player > 1 {
            return CardSet::default();
        }
        let effect = self.core.reason_effect.get();
        let mzonecheck = |card: CardId| {
            (!check_level || self.get_level(card) > 0)
                && self.is_affect_by_effect(card, effect)
                && self.is_releasable_by_effect(card, effect)
        };
        let releasable = |card: CardId| self.is_releasable_by_nonsummon(card, player, REASON_EFFECT);
        let own = self.player[player].list_mzone.iter().flatten().copied();
        let mut material: CardSet = own.filter(|&card| mzonecheck(card) && releasable(card)).collect();
        for card in self.player[1 - player].list_mzone.iter().flatten().copied() {
            if self.cards[card].is_position(POS_FACEUP)
                && mzonecheck(card)
                && releasable(card)
                && self.is_affected_by_effect(card, EFFECT_EXTRA_RELEASE).is_some()
            {
                material.insert(card);
            }
        }
        let hand = self.player[player].list_hand.iter().copied();
        material.merge(
            hand.filter(|&card| self.cards[card].data.card_type & TYPE_MONSTER != 0 && releasable(card))
                .collect(),
        );
        if self.may_have_effect(EFFECT_EXTRA_RITUAL_MATERIAL) {
            let info = &self.player[player];
            let overlay = info.list_mzone.iter().flatten().flat_map(|&card| self.xyz_materials(card));
            let lists = info.list_grave.iter().chain(&info.list_main).chain(&info.list_extra).copied();
            unported!(
                self,
                lists
                    .chain(overlay)
                    .find_map(|card| self.is_affected_by_effect(card, EFFECT_EXTRA_RITUAL_MATERIAL))
            );
        }
        material
    }

    // field::ritual_release without release_deck, which no ported card passes.
    pub fn release_ritual_material(&mut self, g: CardSet) -> Script<()> {
        let (mut rel, mut rem, mut tograve) = (CardSet::default(), CardSet::default(), CardSet::default());
        for card in g {
            match self.cards[card].current.location {
                LOCATION_GRAVE => rem.insert(card),
                location if location & (LOCATION_OVERLAY | LOCATION_EXTRA | LOCATION_DECK) != 0 => tograve.insert(card),
                _ => rel.insert(card),
            }
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        let reason = REASON_RITUAL + REASON_EFFECT + REASON_MATERIAL;
        let send = |duel: &mut Duel, targets, destination| {
            duel.send_to(
                targets,
                reason_effect,
                reason,
                reason_player,
                PLAYER_NONE,
                destination,
                0,
                POS_FACEUP as u8,
                false,
            );
        };
        send(self, tograve, LOCATION_GRAVE);
        self.field_release(rel, reason_effect, reason, reason_player);
        send(self, rem, LOCATION_REMOVED);
        self.yield_script()
    }

    // Lua passes a card or a group.
    pub fn set_selected_card(&self, g: impl Into<CardSet>) {
        self.core.must_select_cards.set(g.into());
    }

    // The SelectFusion unit in one: the material effects that accept the materials, then the operation of the chosen
    // one, which sets core.fusion_materials. With none, the core leaves core.fusion_materials as it was and returns
    // that; the choice between several material effects comes with the cards that have more than one.
    pub fn select_fusion_material(&mut self, player: u8, card: CardId, g: CardSet, gc: Option<CardSet>, chkf: u64) -> Script<CardSet> {
        if player > 1 {
            return Ok(CardSet::default());
        }
        let effects: EffectSet = self.fusion_materials_valid(card, Some(g), gc, chkf).collect();
        unported!(self, effects.get(1));
        if let Some(&effect) = effects.first() {
            self.core.fusion_materials.clear();
            let event = Event {
                event_cards: Some(g),
                reason_effect: Some(effect),
                reason_player: player,
                ..Event::default()
            };
            let procedure = Procedure {
                card,
                must: gc,
                chkf,
                ..Procedure::default()
            };
            self.execute_procedure(ExecuteKind::Operation, effect, player, event, Some(procedure));
        }
        self.yield_script()?;
        Ok(self.core.fusion_materials)
    }

    pub fn set_fusion_material(&mut self, g: CardSet) {
        self.core.fusion_materials = g;
    }

    pub fn get_fusion_check_additional(&self) -> Option<FusionCheck> {
        self.fusion_check_additional
    }

    pub fn set_fusion_check_additional(&mut self, check: Option<FusionCheck>) {
        self.fusion_check_additional = check;
    }

    pub fn get_ritual_summoning_level(&self) -> Option<u32> {
        self.ritual_summoning_level.get()
    }

    pub fn set_ritual_summoning_level(&self, level: Option<u32>) {
        self.ritual_summoning_level.set(level);
    }

    pub fn special_summon_step(
        &mut self,
        target: CardId,
        sumtype: u32,
        sumplayer: u8,
        target_player: u8,
        nocheck: bool,
        nolimit: bool,
        positions: u32,
    ) -> Script<bool> {
        if sumplayer >= PLAYER_NONE {
            return Ok(false);
        }
        self.field_special_summon_step(target, sumtype, sumplayer, target_player, nocheck, nolimit, positions as u8, 0xff);
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }

    pub fn special_summon_complete(&mut self) -> Script<i32> {
        if self.core.special_summoning.is_empty() && self.core.ss_tograve_set.is_empty() {
            self.core.operated_set.clear();
            return Ok(0);
        }
        self.field_special_summon_complete(self.core.reason_effect.get(), self.core.reason_player.get());
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn get_attacker(&self) -> Option<CardId> {
        self.core.attacker
    }

    // The AttackDisable unit in one. core.effect_damage_step is set only by Duel.CalculateDamage, which no port calls.
    pub fn negate_attack(&mut self) -> Script<bool> {
        let reason_effect = self.core.reason_effect.get();
        let Some(attacker) = self.core.attacker.filter(|&attacker| {
            self.cards[attacker].fieldid_r == self.core.pre_field[0]
                && self.cards[attacker].current.location == LOCATION_MZONE
                && self.is_capable_attack(attacker)
                && self.is_affect_by_effect(attacker, reason_effect)
                && self.is_affected_by_effect(attacker, EFFECT_UNSTOPPABLE_ATTACK).is_none()
        }) else {
            return Ok(false);
        };
        let effect = self.new_effect(CardId::default());
        let peffect = &mut self.effects[effect];
        peffect.code = EFFECT_ATTACK_DISABLED;
        peffect.effect_type = EFFECT_TYPE_SINGLE;
        self.card_add_effect(attacker, effect);
        self.cards[attacker].set_status(STATUS_ATTACK_CANCELED, true);
        let reason_player = self.core.reason_player.get();
        self.raise_event(
            Some(attacker.into()),
            EVENT_ATTACK_DISABLED,
            reason_effect,
            0,
            reason_player,
            PLAYER_NONE,
            0,
        );
        self.process_instant_event();
        self.yield_script()?;
        Ok(true)
    }
}
