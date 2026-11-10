// Port of libduel.cpp. Functions that yield in the core run their units before they return.
use carddata::{
    OPCODE_ADD, OPCODE_ALLOW_ALIASES, OPCODE_ALLOW_TOKENS, OPCODE_AND, OPCODE_BAND, OPCODE_BNOT, OPCODE_BOR, OPCODE_BXOR, OPCODE_DIV,
    OPCODE_ISATTRIBUTE, OPCODE_ISCODE, OPCODE_ISRACE, OPCODE_ISSETCARD, OPCODE_ISTYPE, OPCODE_LSHIFT, OPCODE_MUL, OPCODE_NEG, OPCODE_NOT,
    OPCODE_OR, OPCODE_RSHIFT, OPCODE_SUB,
};

use crate::card::{CardId, CardSet, LocInfo, StaticData};
use crate::chain::ExecuteKind;
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{
    ChainLimit, ChainLimitFn, Container, CounterFilter, CustomCounter, EffectId, EffectSet, Event, OpTarget, OperationFn, Value,
};

use crate::field::FromEx;
use crate::operations::{PayLpCost, TossCoin, TossDice};
use crate::processor::{
    AnnounceAttribute, AnnounceCard, AnnounceNumber, AnnounceRace, SelectCard, SelectOption, SelectPlace, SelectPosition, SelectYesNo,
    SortDeck,
};
use crate::script::{FusionCheck, Procedure, Script};

// What Duel.GetChainInfo pushes for one CHAININFO flag.
#[derive(Debug, Clone, Copy)]
pub enum ChainInfo {
    Int(i64),
    Effect(EffectId),
    Cards(Option<CardSet>),
    Setcodes([u16; 5]),
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

int_chain_info!(u8, u16, u32, u64, i32, i64);

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

// CHAININFO_TRIGGERING_SETCODES, a table of the archetypes; unused slots are 0.
impl FromChainInfo for Option<[u16; 5]> {
    fn from_chain_info(info: ChainInfo) -> Self {
        match info {
            ChainInfo::Setcodes(setcodes) => Some(setcodes),
            _ => None,
        }
    }
}

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

// The arguments Duel.CheckReleaseGroup and Duel.SelectReleaseGroup take in their long form after the counts.
#[derive(Debug, Clone, Copy)]
pub struct ReleaseOptions {
    pub use_hand: bool,
    pub check_field: bool,
    pub to_check: Option<CardId>,
    pub toplayer: u8,
    pub zone: u32,
    pub use_oppo: bool,
    pub exception: Option<CardId>,
    pub exception_group: Option<CardSet>,
}

impl ReleaseOptions {
    // Where Lua leaves the arguments out: the player's own zones, no field check.
    pub fn new(player: u8) -> Self {
        Self {
            use_hand: false,
            check_field: false,
            to_check: None,
            toplayer: player,
            zone: 0xff,
            use_oppo: false,
            exception: None,
            exception_group: None,
        }
    }
}

// The card_data fields Duel.IsPlayerCanSpecialSummonMonster gives the temp card over the database's data of its code;
// None (Lua nil) keeps the database's.
#[derive(Debug, Clone, Copy, Default)]
pub struct MonsterStats {
    pub setcode: Option<u16>,
    pub card_type: Option<u32>,
    pub attack: Option<i32>,
    pub defense: Option<i32>,
    pub level: Option<u32>,
    pub race: Option<u64>,
    pub attribute: Option<u32>,
}

impl MonsterStats {
    pub const fn new(setcode: u16, card_type: u32, attack: i32, defense: i32, level: u32, race: u64, attribute: u32) -> Self {
        Self {
            setcode: Some(setcode),
            card_type: Some(card_type),
            attack: Some(attack),
            defense: Some(defense),
            level: Some(level),
            race: Some(race),
            attribute: Some(attribute),
        }
    }
}

const CHAININFO_TRIGGERING_EFFECT: u32 = 1;
const CHAININFO_TRIGGERING_PLAYER: u32 = 2;
const CHAININFO_TRIGGERING_CONTROLER: u32 = 3;
const CHAININFO_TRIGGERING_LOCATION: u32 = 4;
const CHAININFO_TRIGGERING_LOCATION_SYMBOLIC: u32 = 5;
const CHAININFO_TRIGGERING_SEQUENCE: u32 = 6;
const CHAININFO_TRIGGERING_POSITION: u32 = 16;
const CHAININFO_TRIGGERING_CODE: u32 = 17;
const CHAININFO_TRIGGERING_TYPE: u32 = 19;
const CHAININFO_TRIGGERING_ATTRIBUTE: u32 = 22;
const CHAININFO_TRIGGERING_RACE: u32 = 23;
const CHAININFO_TRIGGERING_ATTACK: u32 = 24;
const CHAININFO_TRIGGERING_SUMMON_LOCATION: u32 = 27;
const CHAININFO_TARGET_CARDS: u32 = 8;
const CHAININFO_TARGET_PLAYER: u32 = 9;
const CHAININFO_TARGET_PARAM: u32 = 10;
const CHAININFO_DISABLE_REASON: u32 = 11;
const CHAININFO_DISABLE_PLAYER: u32 = 12;
const CHAININFO_CHAIN_ID: u32 = 13;
const CHAININFO_TRIGGERING_SETCODES: u32 = 30;

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
        self.get_current_chain_full(false)
    }

    pub fn get_current_chain_full(&self, real: bool) -> u8 {
        if real {
            self.core.real_chain_count as u8
        } else {
            self.core.current_chain.len() as u8
        }
    }

    pub fn get_lp(&self, player: u8) -> i32 {
        if player > 1 {
            return 0;
        }
        self.player[player].lp
    }

    pub fn set_lp(&mut self, player: u8, lp: i32) {
        if player > 1 {
            return;
        }
        let lp = lp.max(0);
        self.player[player].lp = lp;
        self.trace.message(MSG_LPUPDATE).u8(player).u32(lp as u32);
    }

    pub fn negate_summon(&mut self, targets: impl Into<CardSet>) {
        let targets = targets.into();
        let mut sumplayer = PLAYER_NONE;
        for card in targets {
            let target = &mut self.cards[card];
            sumplayer = target.summon.player;
            target.set_status(STATUS_SUMMONING, false);
            target.set_status(STATUS_SUMMON_DISABLED, true);
            if target.summon.summon_type & SUMMON_TYPE_PENDULUM != SUMMON_TYPE_PENDULUM {
                target.set_status(STATUS_PROC_COMPLETE, false);
            }
        }
        let event_code = [
            (EVENT_SUMMON, EVENT_SUMMON_NEGATED),
            (EVENT_FLIP_SUMMON, EVENT_FLIP_SUMMON_NEGATED),
            (EVENT_SPSUMMON, EVENT_SPSUMMON_NEGATED),
        ]
        .into_iter()
        .find(|&(event, _)| self.field_check_event(event).is_some())
        .map_or(0, |(_, negated)| negated);
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.field_raise_event(Some(targets), event_code, reason_effect, REASON_EFFECT, reason_player, sumplayer, 0);
        self.process_instant_event();
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

    // Duel.CheckLocation; the core returns nothing for a player above 1.
    pub fn check_location(&self, player: u8, location: u32, sequence: u32) -> bool {
        player <= 1 && self.is_location_useable(player, location as u16, sequence)
    }

    // Duel.GetDrawCount; the core returns nothing for a player above 1.
    pub fn get_draw_count(&self, player: u8) -> i32 {
        if player > 1 { 0 } else { self.field_get_draw_count(player) }
    }

    // Duel.IsPlayerCanRelease without a card.
    pub fn is_player_can_release(&self, player: u8) -> bool {
        player <= 1 && self.is_player_can_action(player, EFFECT_CANNOT_RELEASE)
    }

    // Duel.IsPlayerCanSummon without a card.
    pub fn is_player_can_summon(&self, player: u8) -> bool {
        player <= 1 && self.is_player_can_action(player, EFFECT_CANNOT_SUMMON)
    }

    pub fn is_player_can_additional_summon(&self, player: u8) -> bool {
        player <= 1 && self.core.extra_summon[player] == 0
    }

    pub fn is_player_can_sendto_hand(&self, player: u8) -> bool {
        player <= 1 && self.is_player_can_action(player, EFFECT_CANNOT_TO_HAND)
    }

    pub fn is_player_can_remove(&self, player: u8, card: Option<CardId>) -> bool {
        self.is_player_can_remove_full(player, card, REASON_EFFECT)
    }

    pub fn is_player_can_remove_full(&self, player: u8, card: Option<CardId>, reason: u32) -> bool {
        player <= 1
            && match card {
                Some(card) => self.field_is_player_can_remove(player, card, reason),
                None => self.is_player_can_action(player, EFFECT_CANNOT_REMOVE),
            }
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

    pub fn equip_complete(&mut self) -> Script<()> {
        let equips = self.core.equiping_cards;
        let mut targets = CardSet::default();
        for equip in equips {
            if self.cards[equip].is_position(POS_FACEUP) {
                self.enable_field_effect(equip, true);
            }
            if let Some(target) = self.cards[equip].equiping_target {
                targets.insert(target);
            }
        }
        if targets.is_empty() {
            return Ok(());
        }
        self.adjust_instant();
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        for target in targets {
            self.raise_single_event_with(target, Some(equips), EVENT_EQUIP, reason_effect, 0, reason_player, PLAYER_NONE, 0);
        }
        self.field_raise_event(Some(equips), EVENT_EQUIP, reason_effect, 0, reason_player, PLAYER_NONE, 0);
        self.core.hint_timing[0] |= TIMING_EQUIP;
        self.core.hint_timing[1] |= TIMING_EQUIP;
        self.process_single_event();
        self.process_instant_event();
        self.yield_script()
    }

    // Duel.GetReleaseGroup, which proc_workaround.lua replaces.
    pub fn get_release_group_core(&self, player: u8, hand: bool, oppo: bool, reason: u32) -> CardSet {
        let mut group = CardSet::default();
        if player <= 1 {
            self.get_release_list(player, &mut group, hand, None, None, None, oppo, reason);
        }
        group
    }

    // Duel.CheckReleaseGroup in its long form (the core ignores max).
    pub fn check_release_group_full(&self, player: u8, f: impl Fn(CardId, &Duel) -> bool, min: usize, options: ReleaseOptions) -> bool {
        if player > 1 {
            return false;
        }
        let result = self.check_release_list(player, min, &f, &options, REASON_EFFECT);
        self.core.must_select_cards.set(CardSet::default());
        result
    }

    // Duel.SelectReleaseGroup in its long form; the empty group stands for nil, which a canceled selection returns. The
    // Extra Deck and opponent's monsters that EFFECT_EXTRA_RELEASE* allow come with the cards that register them.
    pub fn select_release_group_full(
        &mut self,
        player: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        (min, max): (u8, u8),
        cancelable: bool,
        options: ReleaseOptions,
    ) -> Script<CardSet> {
        if player > 1 {
            return Ok(CardSet::default());
        }
        let mut release_cards = CardSet::default();
        self.get_release_list(
            player,
            &mut release_cards,
            options.use_hand,
            Some(&f),
            options.exception,
            options.exception_group,
            options.use_oppo,
            REASON_EFFECT,
        );
        let mut must_choose_one = None;
        if options.check_field {
            let zone = options.zone
                & (0x1f & self.get_forced_zones(options.to_check, options.toplayer, LOCATION_MZONE, player, LOCATION_REASON_TOFIELD));
            let (ct, _) = self.get_useable_count(
                options.to_check,
                options.toplayer,
                LOCATION_MZONE,
                player,
                LOCATION_REASON_TOFIELD,
                zone,
            );
            if ct < i32::from(min) {
                must_choose_one =
                    Some(release_cards.filter(self, |card, duel| duel.is_release_in_zone(card, options.toplayer, zone), None));
            }
        }
        self.return_cards.clear();
        let Some(must_choose) = must_choose_one else {
            let allmust = release_cards.is_empty() || max == 0;
            self.hint(HINT_SELECTMSG, player, 500);
            self.select_cards(player, release_cards, cancelable, if allmust { 0 } else { min }, max)?;
            let selected = self.returned_cards();
            return Ok(if selected.len() < usize::from(min) {
                CardSet::default()
            } else {
                selected
            });
        };
        let mut operated = CardSet::default();
        loop {
            let must_chosen = !operated.intersection(must_choose).is_empty();
            let finishable = operated.len() >= usize::from(min) && must_chosen;
            let curmax = i32::from(max) - operated.len() as i32 - i32::from(!must_chosen);
            let diff = if curmax > 0 {
                release_cards
            } else if !must_chosen {
                must_choose
            } else {
                CardSet::default()
            };
            self.hint(HINT_SELECTMSG, player, 500);
            let picked = diff.difference(operated).select_unselect(
                self,
                Some(operated),
                player,
                finishable,
                finishable || operated.is_empty(),
                min,
                max,
            )?;
            let Some(card) = picked else {
                return Ok(operated);
            };
            if operated.contains(card) {
                operated.remove(card);
            } else {
                operated.insert(card);
            }
            if operated.len() == usize::from(max) {
                return Ok(operated);
            }
        }
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

    pub fn is_player_can_discard_deck_as_cost(&self, player: u8, count: usize) -> bool {
        if !self.is_player_can_discard_deck(player, count) {
            return false;
        }
        let Some(&topcard) = self.player[player].list_main.last() else {
            return true;
        };
        if count == 1 && self.cards[topcard].is_position(POS_FACEUP) {
            return self.is_capable_cost_to_grave(topcard, player);
        }
        let cant_remove_s = !self.is_player_can_action(player, EFFECT_CANNOT_REMOVE);
        let cant_remove_o = !self.is_player_can_action(1 - player, EFFECT_CANNOT_REMOVE);
        self.filter_field_effect(EFFECT_TO_GRAVE_REDIRECT).into_iter().all(|effect| {
            let (redirect, p, peffect) = (
                self.get_value(effect, topcard) as u32,
                self.get_handler_player(effect),
                &self.effects[effect],
            );
            let skipped = redirect & u32::from(LOCATION_REMOVED) != 0
                && (self.is_affected_by_effect(topcard, EFFECT_CANNOT_REMOVE).is_some()
                    || if p == player { cant_remove_s } else { cant_remove_o });
            let range = if p == player { peffect.s_range } else { peffect.o_range };
            skipped || range & u16::from(LOCATION_DECK) == 0
        })
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
        self.get_location_count_full(player, location, self.core.reason_player.get(), LOCATION_REASON_TOFIELD, 0xff)
    }

    pub fn get_location_count_full(&self, player: u8, location: u32, uplayer: u8, reason: u32, zone: u32) -> i32 {
        if player > 1 {
            return 0;
        }
        self.get_useable_count(None, player, location as u8, uplayer, reason, zone).0
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

    pub fn get_first_matching_card(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> Option<CardId> {
        let mut first = None;
        let mut group = CardSet::default();
        let exception = exception.unwrap_or_default();
        self.filter_matching_card(
            &mut |duel, card| {
                let passes = filter(card, duel);
                if passes {
                    first = Some(card);
                }
                passes
            },
            player,
            location1,
            location2,
            &mut group,
            &exception,
            1,
            false,
        );
        first
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

    pub fn get_target_count(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> usize {
        self.matching_cards(filter, player, location1, location2, exception, 0, true)
            .1
            .len()
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

    pub fn clear_target_card(&mut self) {
        if let Some(chain) = self.get_chain(0)
            && let Some(targets) = &mut chain.target_cards
        {
            targets.clear();
        }
    }

    pub fn clear_operation_info(&mut self, chain_count: u8) {
        if let Some(chain) = self.get_chain(chain_count) {
            chain.opinfos.clear();
            chain.possibleopinfos.clear();
        }
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

    // field::change_target: the index of the chain link, which counts past the chain or before it take the newest.
    fn chain_link_index(&self, chain_count: u8) -> Option<usize> {
        let len = self.core.current_chain.len();
        (len > 0).then(|| {
            if usize::from(chain_count) > len || chain_count < 1 {
                len - 1
            } else {
                usize::from(chain_count) - 1
            }
        })
    }

    pub fn change_target_card(&mut self, chain_count: u8, targets: CardSet) {
        let Some(index) = self.chain_link_index(chain_count) else { return };
        let Some(chain) = self.core.current_chain.get_mut(index) else {
            return;
        };
        let Some(old_targets) = chain.target_cards else { return };
        chain.target_cards = Some(targets);
        let (effect, chain_id) = (chain.triggering_effect, chain.chain_id);
        for card in old_targets {
            self.release_relation(card, effect, chain_id);
        }
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

    pub fn change_chain_operation(&mut self, chain_count: u8, operation: OperationFn) {
        let Some(index) = self.chain_link_index(chain_count) else { return };
        let Some(chain) = self.core.current_chain.get_mut(index) else {
            return;
        };
        chain.replace_op = Some(operation);
        let effect = chain.triggering_effect;
        let handler = self.get_handler(effect);
        if self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE != 0 && self.cards[handler].current.location == LOCATION_SZONE {
            self.cards[handler].set_status(STATUS_LEAVE_CONFIRMED, true);
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
            CHAININFO_TRIGGERING_POSITION => int(chain.triggering_position.into()),
            CHAININFO_TRIGGERING_CODE => int(chain.triggering_state.code.into()),
            CHAININFO_TRIGGERING_TYPE => int(chain.triggering_state.card_type.into()),
            CHAININFO_TRIGGERING_ATTRIBUTE => int(chain.triggering_state.attribute.into()),
            CHAININFO_TRIGGERING_RACE => int(chain.triggering_state.race as i64),
            CHAININFO_TRIGGERING_ATTACK => int(chain.triggering_state.attack.into()),
            CHAININFO_TRIGGERING_SUMMON_LOCATION => int(chain.triggering_summon_location.into()),
            CHAININFO_TARGET_CARDS => ChainInfo::Cards(chain.target_cards),
            CHAININFO_TARGET_PLAYER => int(chain.target_player.into()),
            CHAININFO_TARGET_PARAM => int(chain.target_param.into()),
            CHAININFO_DISABLE_REASON => chain.disable_reason.map_or(ChainInfo::Cards(None), ChainInfo::Effect),
            CHAININFO_DISABLE_PLAYER => int(chain.disable_player.into()),
            CHAININFO_CHAIN_ID => int(chain.chain_id.into()),
            CHAININFO_TRIGGERING_SETCODES => ChainInfo::Setcodes(chain.triggering_state.setcodes),
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
        self.destroy_full(targets, reason, u32::from(LOCATION_GRAVE), self.core.reason_player.get())
    }

    pub fn destroy_full(&mut self, targets: impl Into<CardSet>, reason: u32, destination: u32, reason_player: u8) -> Script<i32> {
        let reason_effect = self.core.reason_effect.get();
        self.field_destroy(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            PLAYER_NONE,
            destination as u8,
            0,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn send_to_grave(&mut self, targets: impl Into<CardSet>, reason: u32) -> Script<i32> {
        self.send_to_grave_full(targets, reason, PLAYER_NONE, self.core.reason_player.get())
    }

    pub fn send_to_grave_full(&mut self, targets: impl Into<CardSet>, reason: u32, player: u8, reason_player: u8) -> Script<i32> {
        let reason_effect = self.core.reason_effect.get();
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            player,
            LOCATION_GRAVE,
            0,
            POS_FACEUP as u8,
            false,
        );
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn send_to_extra_p(&mut self, targets: impl Into<CardSet>, player: u8, reason: u32) -> Script<i32> {
        self.send_to_extra_p_full(targets, player, reason, self.core.reason_player.get())
    }

    pub fn send_to_extra_p_full(&mut self, targets: impl Into<CardSet>, player: u8, reason: u32, reason_player: u8) -> Script<i32> {
        if player > PLAYER_NONE {
            return Ok(0);
        }
        let reason_effect = self.core.reason_effect.get();
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            player,
            LOCATION_EXTRA,
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
        self.special_summon_full(targets, sumtype, sumplayer, target_player, nocheck, nolimit, positions, 0xff)
    }

    pub fn special_summon_full(
        &mut self,
        targets: impl Into<CardSet>,
        sumtype: u32,
        sumplayer: u8,
        target_player: u8,
        nocheck: bool,
        nolimit: bool,
        positions: u32,
        zone: u32,
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
            zone,
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

    pub fn check_lp_cost(&self, player: u8, lp: u32) -> bool {
        unported!(self, self.filter_player_effect(player, EFFECT_LPCOST_CHANGE, true));
        unported!(self, self.field_effect_range(Container::Continuous, EFFECT_LPCOST_REPLACE));
        player <= 1 && (lp == 0 || lp as i32 <= self.player[player].lp)
    }

    pub fn pay_lp_cost(&mut self, player: u8, cost: u32) -> Script<()> {
        if player > 1 {
            return Ok(());
        }
        self.emplace_process(PayLpCost {
            step: 0,
            playerid: player,
            cost,
        });
        self.yield_script()
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
        let during_to_hand = self.field_check_event(EVENT_TO_HAND).is_some();
        let mut hand = CardSet::default();
        for card in targets {
            self.field_raise_single_event(card, EVENT_CONFIRM, reason_effect, reason, reason_player, revealing_player, 0);
            if during_to_hand && self.cards[card].current.location == LOCATION_HAND {
                self.field_raise_single_event(
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
        self.field_raise_event(
            Some(targets),
            EVENT_CONFIRM,
            reason_effect,
            reason,
            reason_player,
            revealing_player,
            0,
        );
        if !hand.is_empty() {
            self.field_raise_event(
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

    // Duel.ShuffleSetCard: at most 5 face-down cards in one zone of one player's swap places at random.
    pub fn shuffle_set_card(&mut self, targets: impl Into<CardSet>) -> Script<()> {
        let targets: CardSet = targets.into();
        let Some(first) = targets.get_first().map(|card| self.cards[card].current) else {
            return Ok(());
        };
        let (location, tp) = (first.location, first.controler);
        let shuffleable = |card: CardId| {
            let current = self.cards[card].current;
            current.location == location && current.controler == tp && current.position & POS_FACEUP == 0 && current.sequence <= 4
        };
        if targets.len() > 5 || (location != LOCATION_MZONE && location != LOCATION_SZONE) || tp > 1 || !targets.iter().all(shuffleable) {
            return Ok(());
        }
        let mut shuffled: Vec<CardId> = targets.iter().collect();
        let sequences: Vec<u32> = shuffled.iter().map(|&card| self.cards[card].current.sequence).collect();
        for i in (1..shuffled.len()).rev() {
            let swap = self.random.get_next_integer(0, i as i32) as usize;
            shuffled.swap(i, swap);
        }
        let infos: Vec<LocInfo> = targets.iter().map(|card| self.cards.get_info_location(card)).collect();
        for (&card, sequence) in shuffled.iter().zip(sequences) {
            if let Some(slot) = self.player[tp].zone_mut(location, sequence) {
                *slot = Some(card);
            }
            let current = &mut self.cards[card].current;
            current.sequence = sequence;
            let (reason_effect, reason, reason_player) = (current.reason_effect, current.reason, current.reason_player);
            self.field_raise_single_event(card, EVENT_MOVE, reason_effect, reason, reason_player, tp, 0);
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.field_raise_event(Some(targets), EVENT_MOVE, reason_effect, 0, reason_player, tp, 0);
        self.process_single_event();
        self.process_instant_event();
        let materials: Vec<LocInfo> = targets
            .iter()
            .map(|card| match self.xyz_materials(card).is_empty() {
                true => LocInfo::default(),
                false => self.cards.get_info_location(card),
            })
            .collect();
        let mut message = self.trace.message(MSG_SHUFFLE_SET_CARD);
        message.u8(location).u8(targets.len() as u8);
        for info in infos.into_iter().chain(materials) {
            message.loc(info);
        }
        drop(message);
        self.yield_script()
    }

    // The values a script keeps in Lua locals across calls, per duel; 0 until set.
    pub fn get_script_state(&self, key: u64) -> i64 {
        self.script_state
            .iter()
            .find(|(other, _)| *other == key)
            .map_or(0, |&(_, value)| value)
    }

    pub fn get_values_reset(&self) -> Vec<fn(&mut Duel)> {
        self.values_reset.clone()
    }

    pub fn add_values_reset(&mut self, resetfunc: fn(&mut Duel)) -> bool {
        self.values_reset.push(resetfunc);
        self.values_reset.len() == 1
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
    Nil,
}

impl From<Option<CardId>> for ExtraSummon {
    fn from(card: Option<CardId>) -> Self {
        card.map_or(ExtraSummon::Nil, ExtraSummon::Card)
    }
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

    pub fn enable_global_flag(&mut self, flag: u32) {
        self.core.global_flag |= flag;
    }

    pub fn check_phase_activity(&self) -> bool {
        self.core.phase_action
    }

    pub fn is_main_phase(&self) -> bool {
        let phase = self.infos.phase;
        phase == PHASE_MAIN1 || phase == PHASE_MAIN2
    }

    pub fn is_able_to_enter_bp(&self) -> bool {
        let turn_player = self.infos.turn_player;
        (self.is_flag(DUEL_ATTACK_FIRST_TURN)
            || self.infos.turn_id != 1
            || self.is_player_affected_by_effect(turn_player, EFFECT_BP_FIRST_TURN).is_some())
            && self.infos.phase < PHASE_BATTLE_START
            && self.is_player_affected_by_effect(turn_player, EFFECT_CANNOT_BP).is_none()
    }

    pub fn is_battle_phase(&self) -> bool {
        (PHASE_BATTLE_START..=PHASE_BATTLE).contains(&self.infos.phase)
    }

    pub fn is_damage_calculated(&self) -> bool {
        self.core.damage_calculated
    }

    pub fn is_phase(&self, phase: u32) -> bool {
        u32::from(self.infos.phase) == phase
    }

    pub fn change_position(&mut self, targets: impl Into<CardSet>, position: u32) -> Script<i32> {
        self.change_position_full(targets, [position; 4], false)
    }

    // `positions` are the Lua's au, ad, du and dd.
    pub fn change_position_full(&mut self, targets: impl Into<CardSet>, positions: [u32; 4], no_flip_effect: bool) -> Script<i32> {
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        let flag = if no_flip_effect { NO_FLIP_EFFECT } else { 0 };
        self.field_change_positions(targets.into(), reason_effect, reason_player, positions, flag);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    pub fn remove(&mut self, targets: impl Into<CardSet>, position: u32, reason: u32) -> Script<i32> {
        self.remove_full(targets, position, reason, PLAYER_NONE, None)
    }

    pub fn remove_full(
        &mut self,
        targets: impl Into<CardSet>,
        position: u32,
        reason: u32,
        player: u8,
        reason_player: Option<u8>,
    ) -> Script<i32> {
        let reason_effect = self.core.reason_effect.get();
        let reason_player = reason_player.unwrap_or_else(|| self.core.reason_player.get());
        self.send_to(
            targets.into(),
            reason_effect,
            reason,
            reason_player,
            player,
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
            self.field_raise_single_event(card, EVENT_MOVE, reason_effect, 0, reason_player, player, 0);
        }
        self.field_raise_event(
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

    // The top `count` cards are shown to both players and returned.
    pub fn confirm_decktop(&mut self, player: u8, count: usize) -> Script<CardSet> {
        if player > 1 {
            return Ok(CardSet::default());
        }
        let deck = &self.player[player].list_main;
        let count = count.min(deck.len());
        unported!(self, (count < deck.len() && self.core.deck_reversed).then_some(player));
        let mut message = self.trace.message(MSG_CONFIRM_DECKTOP);
        message.u8(player).u32(count as u32);
        for &card in deck.iter().rev().take(count) {
            let card = &self.cards[card];
            message
                .u32(card.data.code)
                .u8(card.current.controler)
                .u8(card.current.location)
                .u32(card.current.sequence);
        }
        drop(message);
        self.yield_script()?;
        Ok(self.get_decktop_group(player, count))
    }

    // Duel.MoveSequence for a card in the Deck or a Monster Zone; the sequences of other locations come with the cards that move them.
    pub fn move_sequence(&mut self, card: CardId, sequence: u8) -> Script<bool> {
        let location = self.cards[card].current.location;
        let invalid_mzone = sequence > if self.is_flag(DUEL_EMZONE) { 6 } else { 4 };
        unported!(
            self,
            (location != LOCATION_DECK && (location != LOCATION_MZONE || invalid_mzone)).then_some(card)
        );
        let reason_effect = self.core.reason_effect.get();
        if !self.is_affect_by_effect(card, reason_effect) {
            return Ok(false);
        }
        let player = self.cards[card].current.controler;
        let moved = self.move_card(player, card, location, sequence, false);
        if moved {
            let reason_player = self.core.reason_player.get();
            self.field_raise_single_event(card, EVENT_MOVE, reason_effect, 0, reason_player, player, 0);
            self.field_raise_event(Some(card.into()), EVENT_MOVE, reason_effect, 0, reason_player, player, 0);
            self.process_single_event();
            self.process_instant_event();
        }
        self.yield_script()?;
        Ok(moved)
    }

    pub fn get_operated_group(&self) -> CardSet {
        self.core.operated_set
    }

    pub fn break_effect(&mut self) -> Script<()> {
        self.field_break_effect(true);
        self.field_raise_event(None, EVENT_BREAK_EFFECT, None, 0, PLAYER_NONE, PLAYER_NONE, 0);
        self.process_instant_event();
        self.yield_script()
    }

    pub fn raise_event(
        &mut self,
        targets: impl Into<CardSet>,
        code: u32,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        event_player: u8,
        event_value: u32,
    ) -> Script<()> {
        self.field_raise_event(
            Some(targets.into()),
            code,
            reason_effect,
            reason,
            reason_player,
            event_player,
            event_value,
        );
        self.process_instant_event();
        self.yield_script()
    }

    pub fn raise_single_event(
        &mut self,
        card: CardId,
        code: u32,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        event_player: u8,
        event_value: u32,
    ) -> Script<()> {
        self.field_raise_single_event(card, code, reason_effect, reason, reason_player, event_player, event_value);
        self.process_single_event();
        self.yield_script()
    }

    // `capture` is the value the Lua closure captures, which the function receives last (0 for none).
    pub fn set_chain_limit(&mut self, function: ChainLimitFn, capture: i64) {
        let player = self.core.reason_player.get();
        self.core.chain_limit.push(ChainLimit { function, capture, player });
    }

    pub fn set_chain_limit_till_chain_end(&mut self, function: ChainLimitFn, capture: i64) {
        let player = self.core.reason_player.get();
        self.core.chain_limit_p.push(ChainLimit { function, capture, player });
    }

    pub fn sort_decktop(&mut self, sort_player: u8, target_player: u8, count: u16) -> Script<()> {
        self.sort_deck(sort_player, target_player, count, false)
    }

    pub fn sort_deckbottom(&mut self, sort_player: u8, target_player: u8, count: u16) -> Script<()> {
        self.sort_deck(sort_player, target_player, count, true)
    }

    fn sort_deck(&mut self, sort_player: u8, target_player: u8, count: u16, bottom: bool) -> Script<()> {
        if sort_player > 1 || target_player > 1 || !(1..=64).contains(&count) {
            return Ok(());
        }
        self.emplace_process(SortDeck {
            sort_player,
            target_player,
            count,
            bottom,
            ..SortDeck::default()
        });
        self.yield_script()
    }

    pub fn get_control(&mut self, targets: impl Into<CardSet>, player: u8) -> Script<i32> {
        self.get_control_full(targets, player, 0, 0, 0xff, self.core.reason_player.get())
    }

    pub fn get_control_full(
        &mut self,
        targets: impl Into<CardSet>,
        player: u8,
        reset_phase: u16,
        reset_count: u8,
        zone: u32,
        chose_player: u8,
    ) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        self.emplace_process(crate::operations::GetControl {
            reason_effect: self.core.reason_effect.get(),
            chose_player,
            targets: targets.into(),
            playerid: player,
            reset_phase: reset_phase & 0x3ff,
            reset_count,
            zone,
            ..Default::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }

    // Duel.SwapControl for two cards: whether the swap happened.
    pub fn swap_control(&mut self, card1: CardId, card2: CardId, reset_phase: u16, reset_count: u8) -> Script<bool> {
        self.emplace_process(crate::operations::SwapControl {
            reason_effect: self.core.reason_effect.get(),
            reason_player: self.core.reason_player.get(),
            targets1: card1.into(),
            targets2: card2.into(),
            reset_phase: reset_phase & 0x3ff,
            reset_count,
            ..Default::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
    }

    // What Card.CheckActivateEffect does to the chain link being built when a script calls it with copy_info.
    pub fn set_current_chain_event(&mut self, event: Event) {
        if let Some(chain) = self.core.current_chain.last_mut() {
            chain.evt = event;
        }
    }

    pub fn adjust_instantly(&mut self, card: Option<CardId>) {
        if let Some(card) = card {
            self.filter_disable_related_cards(card);
        }
        self.adjust_instant();
    }

    // Duel.Readjust; the core reads the reason effect's handler, so it is only called from an effect.
    pub fn readjust(&mut self) -> Script<()> {
        let Some(reason_effect) = self.core.reason_effect.get() else {
            return Ok(());
        };
        let adjcard = reason_effect.get_handler(self);
        let count = self.core.readjust_map.entry(adjcard).or_insert(0);
        *count += 1;
        if *count > 3 {
            let reason_player = self.core.reason_player.get();
            self.send_to(
                adjcard.into(),
                None,
                REASON_RULE,
                reason_player,
                PLAYER_NONE,
                LOCATION_GRAVE,
                0,
                POS_FACEUP as u8,
                false,
            );
            return self.yield_script();
        }
        self.core.re_adjust = true;
        Ok(())
    }

    // Duel.IsEnvironment(code) for both players, in the Field Zone and on the field.
    pub fn is_environment(&self, code: u32) -> bool {
        let enabled = |card: CardId| self.cards[card].is_position(POS_FACEUP) && self.cards[card].get_status(STATUS_EFFECT_ENABLED);
        let environment = |card: Option<CardId>| card.is_some_and(|card| enabled(card) && card.get_code(self) == code);
        let fields = [0, 1].map(|player| self.player[player].list_szone[5]);
        let on_field = [0, 1]
            .into_iter()
            .flat_map(|player| self.player[player].list_szone.into_iter().chain(self.player[player].list_mzone));
        if on_field.into_iter().any(environment) {
            return true;
        }
        let no_field_spell = !fields.into_iter().any(|card| card.is_some_and(enabled));
        unported!(
            self,
            no_field_spell
                .then(|| self.filter_field_effect(EFFECT_CHANGE_ENVIRONMENT))
                .into_iter()
                .flatten()
        );
        false
    }

    pub fn skip_phase(&mut self, player: u8, phase: u32, reset: u32, count: u16) {
        self.skip_phase_full(player, phase, reset, count, 0);
    }

    pub fn skip_phase_full(&mut self, player: u8, phase: u32, reset: u32, count: u16, value: u32) {
        let code = match phase as u16 {
            PHASE_DRAW => EFFECT_SKIP_DP,
            PHASE_STANDBY => EFFECT_SKIP_SP,
            PHASE_MAIN1 => EFFECT_SKIP_M1,
            PHASE_BATTLE => EFFECT_SKIP_BP,
            PHASE_MAIN2 => EFFECT_SKIP_M2,
            PHASE_END => EFFECT_SKIP_EP,
            _ => return,
        };
        if player > 1 {
            return;
        }
        let effect = self.global_effect();
        let peffect = &mut self.effects[effect];
        peffect.effect_owner = player;
        peffect.effect_type = EFFECT_TYPE_FIELD;
        peffect.code = code;
        peffect.reset_flag = (reset & 0x3ff) | RESET_PHASE | RESET_SELF_TURN;
        peffect.flag[0] = EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_PLAYER_TARGET;
        peffect.s_range = 1;
        peffect.o_range = 0;
        peffect.reset_count = count.max(1);
        peffect.value = Value::Int(value as i32);
        self.add_effect(effect, player);
    }

    pub fn disable_shuffle_check(&mut self, disable: bool) {
        self.core.shuffle_check_disabled = disable;
    }

    // `win_player` is 5 until a win is set; the core's game-over check reports it.
    pub fn win(&mut self, player: u8, reason: u8) {
        let cannot_lose = |p| self.is_player_affected_by_effect(p, EFFECT_CANNOT_LOSE_EFFECT).is_some();
        let player = match player {
            0 | 1 if cannot_lose(1 - player) => return,
            0 | 1 => player,
            2 if cannot_lose(0) && cannot_lose(1) => return,
            2 if cannot_lose(0) => 0,
            2 if cannot_lose(1) => 1,
            2 => 2,
            _ => return,
        };
        match (self.core.win_player, player) {
            (5, _) => self.core.win_player = player,
            (0, 1) | (1, 0) => self.core.win_player = 2,
            _ => return,
        }
        self.core.win_reason = reason;
    }

    pub fn shuffle_deck(&mut self, player: u8) -> Script<()> {
        if player > 1 {
            return Ok(());
        }
        self.shuffle(player, LOCATION_DECK);
        self.yield_script()
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

    pub fn select_effect_yes_no(&mut self, player: u8, card: CardId) -> Script<bool> {
        self.select_effect_yes_no_full(player, card, 95)
    }

    pub fn select_effect_yes_no_full(&mut self, player: u8, card: CardId, description: u64) -> Script<bool> {
        if player > 1 {
            return Ok(false);
        }
        self.field_select_effect_yes_no(player, description, card);
        self.yield_script()?;
        Ok(self.returns.at_i32(0) != 0)
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

    // `count` is capped at the races available, which must be among RACE_ALL.
    pub fn announce_race(&mut self, player: u8, count: u8, available: u64) -> Script<u64> {
        if available & !RACE_ALL != 0 {
            self.reach_unported(concat!(file!(), ":", line!(), " (Duel.AnnounceRace raises a Lua error)"));
        }
        self.emplace_process(AnnounceRace {
            playerid: player,
            count: count.min(available.count_ones() as u8),
            available,
            ..AnnounceRace::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_u64(0))
    }

    pub fn announce_attribute(&mut self, player: u8, count: u8, available: u32) -> Script<u32> {
        if available & !ATTRIBUTE_ALL != 0 {
            self.reach_unported(concat!(file!(), ":", line!(), " (Duel.AnnounceAttribute raises a Lua error)"));
        }
        self.emplace_process(AnnounceAttribute {
            playerid: player,
            count: count.min(available.count_ones() as u8),
            available,
            ..AnnounceAttribute::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_u32(0))
    }

    // Duel.AnnounceCoin with its default hint: COIN_HEADS (1) or tails (0).
    pub fn announce_coin(&mut self, player: u8) -> Script<u8> {
        self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(player).u64(552);
        let selected = self.select_option(player, &[60, 61])?;
        Ok(u8::from(selected == 0))
    }

    // The chosen number (a Lua error without any).
    pub fn announce_number(&mut self, player: u8, numbers: &[u64]) -> Script<u64> {
        if numbers.is_empty() {
            self.reach_unported(concat!(file!(), ":", line!(), " (Duel.AnnounceNumber raises a Lua error)"));
        }
        self.core.select_options = numbers.iter().copied().collect();
        self.emplace_process(AnnounceNumber {
            playerid: player,
            ..AnnounceNumber::default()
        });
        self.yield_script()?;
        Ok(usize::try_from(self.returns.at_i32(0))
            .ok()
            .and_then(|index| numbers.get(index))
            .copied()
            .unwrap_or_default())
    }

    // Duel.AnnounceLevel (AnnounceNumberRange) with its default range 1 to 12 and no excluded level.
    pub fn announce_level(&mut self, player: u8) -> Script<u64> {
        self.announce_number(player, &(1..=12).collect::<Vec<_>>())
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
        self.hint_selection_full(targets, true);
    }

    pub fn hint_selection_full(&mut self, targets: impl Into<CardSet>, selection: bool) {
        let targets = targets.into();
        let locations: Vec<_> = targets.iter().map(|card| self.cards.get_info_location(card)).collect();
        let mut message = self.trace.message(if selection { MSG_CARD_SELECTED } else { MSG_BECOME_TARGET });
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
        self.get_location_count_from_ex_full(player, uplayer, excluded, summoned, 0xff)
    }

    pub fn get_location_count_from_ex_full(
        &self,
        player: u8,
        uplayer: u8,
        excluded: Option<CardSet>,
        summoned: impl Into<ExtraSummon>,
        zone: u32,
    ) -> i32 {
        if player > 1 {
            return 0;
        }
        let zones = excluded.map_or_else(|| self.zones(), |excluded| self.zones_without(excluded));
        let summoned = match summoned.into() {
            ExtraSummon::Card(card) => FromEx::card(self, Some(card)),
            ExtraSummon::Type(card_type) => FromEx::card_type(card_type),
            ExtraSummon::Nil => FromEx::card(self, None),
        };
        self.get_useable_count_fromex_in(&zones, summoned, player, uplayer, zone).0
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
        peffect.set_labels(&[0]);
        self.add_effect(effect, player);
        Some(effect)
    }

    pub fn get_flag_effect(&self, player: u8, code: u32) -> usize {
        if player > 1 {
            return 0;
        }
        self.filter_player_effect(player, (code & 0xfff_ffff) | 0x1000_0000, true).len()
    }

    // The label of the first flag effect of the code, as the first of the values Lua gets.
    pub fn get_flag_effect_label(&self, player: u8, code: u32) -> Option<i64> {
        let effect = self
            .get_player_effect(player, (code & 0xfff_ffff) | 0x1000_0000)
            .into_iter()
            .next()?;
        Some(self.effects[effect].labels().first().copied().unwrap_or(0))
    }

    pub fn reset_flag_effect(&mut self, player: u8, code: u32) {
        if player > 1 {
            return;
        }
        let code = (code & 0xfff_ffff) | 0x1000_0000;
        let flagged: Vec<EffectId> = self.field_effects.registry.equal_range(Container::Aura, None, code).collect();
        for effect in flagged {
            if self.effects[effect].is_flag(EFFECT_FLAG_FIELD_ONLY) && self.get_owner_player(effect) == player {
                self.remove_effect(effect);
            }
        }
    }

    pub fn set_flag_effect_label(&mut self, player: u8, code: u32, label: i32) -> bool {
        let Some(effect) = self.get_player_effect(player, (code & 0xfff_ffff) | 0x1000_0000).into_iter().next() else {
            return false;
        };
        self.effects[effect].set_labels(&[i64::from(label)]);
        true
    }

    // Not in the Lua API: false when no effect of this code is registered anywhere (true may be a false alarm), for
    // library shortcuts that hold only without such effects.
    #[inline(always)]
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
    // Only chain, Special Summon and attack counters are counted; the other activities come with the cards that count them.
    pub fn add_custom_activity_counter(&mut self, id: u32, activity: u8, filter: CounterFilter) {
        if ![ACTIVITY_CHAIN, ACTIVITY_SPSUMMON, ACTIVITY_ATTACK].contains(&activity) {
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

    pub fn summon(&mut self, player: u8, card: CardId, ignore_count: bool, procedure: Option<EffectId>) -> Script<()> {
        self.summon_full(player, card, ignore_count, procedure, 0, 0x1f)
    }

    // During a chain the summon waits until the chain link has resolved.
    pub fn summon_full(
        &mut self,
        player: u8,
        card: CardId,
        ignore_count: bool,
        procedure: Option<EffectId>,
        min_tribute: u8,
        zone: u32,
    ) -> Script<()> {
        if self.core.effect_damage_step != 0 || player > 1 {
            return Ok(());
        }
        self.core.summon_cancelable = false;
        self.field_summon(player, card, procedure, ignore_count, min_tribute, zone);
        if !self.core.current_chain.is_empty() {
            self.core.reserved = self.core.subunits.pop().map(Box::new);
            self.core.summoning_card = Some(card);
        }
        self.yield_script()
    }

    pub fn mset(&mut self, player: u8, card: CardId, ignore_count: bool, procedure: Option<EffectId>) -> Script<()> {
        self.mset_full(player, card, ignore_count, procedure, 0, 0x1f)
    }

    // During a chain the Set waits until the chain link has resolved.
    pub fn mset_full(
        &mut self,
        player: u8,
        card: CardId,
        ignore_count: bool,
        procedure: Option<EffectId>,
        min_tribute: u8,
        zone: u32,
    ) -> Script<()> {
        if self.core.effect_damage_step != 0 || player > 1 {
            return Ok(());
        }
        self.core.summon_cancelable = false;
        self.field_mset(player, card, procedure, ignore_count, min_tribute, zone);
        if !self.core.current_chain.is_empty() {
            self.core.reserved = self.core.subunits.pop().map(Box::new);
            self.core.summoning_card = Some(card);
        }
        self.yield_script()
    }

    // Duel.SelectDisableField without all_field: the zones as a mask of the chosen places, nil for no choice.
    pub fn select_disable_field(&mut self, player: u8, count: u8, location1: u16, location2: u16, filter: u32) -> Script<Option<u32>> {
        if player > 1 {
            return Ok(None);
        }
        let mut flag = u32::MAX;
        let mut available = 0u32;
        for (shift, owner, location, wanted) in [
            (0, player, LOCATION_MZONE, location1),
            (8, player, LOCATION_SZONE, location1),
            (16, 1 - player, LOCATION_MZONE, location2),
            (24, 1 - player, LOCATION_SZONE, location2),
        ] {
            if wanted & u16::from(location) != 0 {
                let (useable, unusable) = self.get_useable_count(None, owner, location, PLAYER_NONE, LOCATION_REASON_NONE, 0xff);
                available = available.wrapping_add(useable as u32);
                flag = (flag & !(0xff << shift)) | (unusable << shift);
            }
        }
        flag |= filter | 0xe0e0_e0e0;
        let count = u32::from(count).min(available) as u8;
        if count == 0 {
            return Ok(None);
        }
        self.emplace_process(SelectPlace {
            playerid: player,
            count,
            flag,
            disable_field: true,
            ..SelectPlace::default()
        });
        self.yield_script()?;
        let picked = (0..usize::from(count)).fold(0, |picked, pick| {
            let (select_player, location, sequence) = (
                self.returns.at_u8(3 * pick),
                self.returns.at_u8(3 * pick + 1),
                self.returns.at_u8(3 * pick + 2),
            );
            let shift = u32::from(sequence) + if select_player == player { 0 } else { 16 } + if location == LOCATION_MZONE { 0 } else { 8 };
            picked | 1u32.wrapping_shl(shift)
        });
        Ok(Some(picked))
    }

    pub fn select_position(&mut self, player: u8, card: CardId, positions: u8) -> Script<u32> {
        let code = self.cards[card].data.code;
        self.emplace_process(SelectPosition {
            playerid: player,
            code,
            positions,
            ..SelectPosition::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_u32(0))
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

    pub fn xyz_summon(&mut self, player: u8, card: CardId, must: Option<CardSet>, materials: Option<CardSet>) -> Script<()> {
        self.xyz_summon_full(player, card, must, materials, 0, 0)
    }

    pub fn xyz_summon_full(
        &mut self,
        player: u8,
        card: CardId,
        must: Option<CardSet>,
        materials: Option<CardSet>,
        min: u16,
        max: u16,
    ) -> Script<()> {
        self.spsummon_rule(SUMMON_TYPE_XYZ, player, card, must, materials, min, max)
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

    pub fn assume_reset(&self) {
        self.assumes.borrow_mut().clear();
    }
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
        self.sset_full(player, targets, player, true)
    }

    pub fn sset_full(&mut self, player: u8, targets: impl Into<CardSet>, toplayer: u8, confirm: bool) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        let toplayer = if toplayer > 1 { player } else { toplayer };
        let targets = targets.into();
        if targets.is_empty() {
            return Ok(self.returns.at_i32(0));
        }
        let reason_effect = self.core.reason_effect.get();
        self.sset_group(player, toplayer, targets, confirm, reason_effect);
        self.yield_script()?;
        Ok(self.returns.at_i32(0))
    }
}

impl Duel {
    // The Lua errors of attaching to a material or a card to itself stop the oracle's script; no port does either.
    pub fn overlay_core(&mut self, target: CardId, materials: impl Into<CardSet>, send_materials_to_grave: bool) -> Script<()> {
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

    pub fn check_remove_overlay_card(&self, player: u8, self_side: u8, oppo: u8, count: u16, reason: u32) -> bool {
        self.check_remove_overlay_card_full(player, self_side, oppo, count, reason, None)
    }

    // The sixth argument of Duel.CheckRemoveOverlayCard: only the overlay cards of this group's Xyz Monsters count.
    pub fn check_remove_overlay_card_full(
        &self,
        player: u8,
        self_side: u8,
        oppo: u8,
        count: u16,
        _reason: u32,
        group: Option<CardSet>,
    ) -> bool {
        player <= 1 && self.is_player_can_remove_overlay_card(player, group, self_side, oppo, count)
    }

    pub fn remove_overlay_card(&mut self, player: u8, self_side: u8, oppo: u8, min: u16, max: u16, reason: u32) -> Script<i32> {
        self.remove_overlay_card_full(player, self_side, oppo, min, max, reason, None)
    }

    // The seventh argument of Duel.RemoveOverlayCard: only the overlay cards of this group's Xyz Monsters.
    pub fn remove_overlay_card_full(
        &mut self,
        player: u8,
        self_side: u8,
        oppo: u8,
        min: u16,
        max: u16,
        reason: u32,
        group: Option<CardSet>,
    ) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        self.field_remove_overlay_card(reason, group, player, self_side, oppo, min, max);
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

    pub fn release_ritual_material(&mut self, g: CardSet) -> Script<()> {
        self.release_ritual_material_full(g, false)
    }

    // With `release_deck`, a material in the Deck is released instead of sent to the GY.
    pub fn release_ritual_material_full(&mut self, g: CardSet, release_deck: bool) -> Script<()> {
        let (mut rel, mut rem, mut tograve) = (CardSet::default(), CardSet::default(), CardSet::default());
        let to_grave_types = LOCATION_OVERLAY | LOCATION_EXTRA | if release_deck { 0 } else { LOCATION_DECK };
        for card in g {
            match self.cards[card].current.location {
                LOCATION_GRAVE => rem.insert(card),
                location if location & to_grave_types != 0 => tograve.insert(card),
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
        self.special_summon_step_full(target, sumtype, sumplayer, target_player, nocheck, nolimit, positions, 0xff)
    }

    pub fn special_summon_step_full(
        &mut self,
        target: CardId,
        sumtype: u32,
        sumplayer: u8,
        target_player: u8,
        nocheck: bool,
        nolimit: bool,
        positions: u32,
        zone: u32,
    ) -> Script<bool> {
        if sumplayer >= PLAYER_NONE {
            return Ok(false);
        }
        self.field_special_summon_step(target, sumtype, sumplayer, target_player, nocheck, nolimit, positions as u8, zone);
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

    pub fn get_card_setcode_from_code(&self, code: u32) -> Vec<u16> {
        let data = self.database.and_then(|database| database.get(code));
        data.into_iter()
            .flat_map(|data| data.setcodes)
            .filter(|&setcode| setcode != 0)
            .collect()
    }

    // None (Lua false) for a player out of range.
    pub fn create_token(&mut self, player: u8, code: u32) -> Option<CardId> {
        if player >= PLAYER_NONE {
            return None;
        }
        let Some(card) = self.database.and_then(|database| self.create_card(code, database).ok()) else {
            self.reach_unported(concat!(file!(), ":", line!(), " (a token not in the database, or over MAX_CARDS)"));
            return None;
        };
        let token = &mut self.cards[card];
        token.owner = player;
        token.current.location = 0;
        token.current.controler = player;
        Some(card)
    }

    pub fn is_player_can_special_summon_monster(&mut self, player: u8, code: u32, stats: MonsterStats) -> bool {
        self.is_player_can_special_summon_monster_full(player, code, stats, POS_FACEUP, player, 0)
    }

    // The core's field::temp_card takes the stats while the check runs.
    pub fn is_player_can_special_summon_monster_full(
        &mut self,
        player: u8,
        code: u32,
        stats: MonsterStats,
        pos: u32,
        toplayer: u8,
        sumtype: u32,
    ) -> bool {
        if player > 1 {
            return false;
        }
        let mut data = self.database.and_then(|database| database.get(code)).copied().unwrap_or_default();
        data.code = code;
        data.alias = 0;
        let setcodes = &mut data.setcodes;
        if let Some(setcode) = stats.setcode.filter(|&setcode| setcode != 0 && !setcodes.contains(&setcode))
            && let Some(slot) = setcodes.iter_mut().take(4).find(|slot| **slot == 0)
        {
            *slot = setcode;
        }
        data.card_type = stats.card_type.unwrap_or(data.card_type);
        data.attack = stats.attack.unwrap_or(data.attack);
        data.defense = stats.defense.unwrap_or(data.defense);
        data.level = stats.level.unwrap_or(data.level);
        data.race = stats.race.unwrap_or(data.race);
        data.attribute = stats.attribute.unwrap_or(data.attribute);
        let temp_card = CardId::default();
        self.cards[temp_card].data = StaticData::interned(data);
        let reason_effect = self.core.reason_effect.get();
        let result = self.is_player_can_spsummon(reason_effect, sumtype, pos as u8, player, toplayer, temp_card);
        self.cards[temp_card].data = StaticData::default();
        result
    }

    // Duel.IsPlayerCanSpecialSummon with a card.
    pub fn is_player_can_special_summon(&self, player: u8, sumtype: u32, sumpos: u32, toplayer: u8, card: CardId) -> bool {
        player <= 1 && self.is_player_can_spsummon(self.core.reason_effect.get(), sumtype, sumpos as u8, player, toplayer, card)
    }

    // Duel.IsPlayerCanSpecialSummon(player) with that one argument.
    pub fn is_player_can_special_summon_any(&self, player: u8) -> bool {
        player <= 1 && self.is_player_can_action(player, EFFECT_CANNOT_SPECIAL_SUMMON) && self.is_player_can_spsummon_count(player, 1)
    }

    pub fn is_player_can_special_summon_count(&self, player: u8, count: u32) -> bool {
        player <= 1 && self.is_player_can_spsummon_count(player, count)
    }

    pub fn get_activity_count(&self, player: u8, activity: u8) -> u32 {
        if player > 1 {
            return 0;
        }
        let core = &self.core;
        u32::from(match activity {
            ACTIVITY_SUMMON => core.summon_state_count[player],
            ACTIVITY_NORMALSUMMON => core.normalsummon_state_count[player],
            ACTIVITY_SPSUMMON => core.spsummon_state_count[player],
            ACTIVITY_FLIPSUMMON => core.flipsummon_state_count[player],
            ACTIVITY_ATTACK => core.attack_state_count[player],
            ACTIVITY_BATTLE_PHASE => core.battle_phase_count[player],
            _ => {
                self.reach_unported(concat!(file!(), ":", line!(), " (Duel.GetActivityCount raises a Lua error)"));
                0
            }
        })
    }

    // The results, 1 for heads; none for a player out of range or no coins.
    pub fn toss_coin(&mut self, player: u8, count: u8) -> Script<Vec<u8>> {
        if player > 1 || count == 0 {
            return Ok(Vec::new());
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.emplace_process(TossCoin {
            reason_effect,
            reason_player,
            playerid: player,
            count: count.min(5),
            ..TossCoin::default()
        });
        self.yield_script()?;
        Ok(self.core.coin_results.to_vec())
    }

    pub fn toss_dice(&mut self, player: u8, count: u8) -> Script<Vec<u8>> {
        self.toss_dice_full(player, count, 0)
    }

    // The player's dice, then the opponent's.
    pub fn toss_dice_full(&mut self, player: u8, count1: u8, count2: u8) -> Script<Vec<u8>> {
        if player > 1 {
            return Ok(Vec::new());
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.emplace_process(TossDice {
            reason_effect,
            reason_player,
            playerid: player,
            count1,
            count2,
            ..TossDice::default()
        });
        self.yield_script()?;
        Ok(self.core.dice_results.to_vec())
    }

    pub fn get_coin_result(&self) -> &[u8] {
        &self.core.coin_results
    }

    pub fn get_dice_result(&self) -> &[u8] {
        &self.core.dice_results
    }

    pub fn set_coin_result(&mut self, results: &[u8]) {
        if results.len() != self.core.coin_results.len() {
            self.reach_unported(concat!(file!(), ":", line!(), " (Duel.SetCoinResult raises a Lua error)"));
            return;
        }
        self.core.coin_results = results.iter().map(|&result| u8::from(result != 0)).collect();
    }

    pub fn set_dice_result(&mut self, results: &[u8]) {
        if results.len() != self.core.dice_results.len() {
            self.reach_unported(concat!(file!(), ":", line!(), " (Duel.SetDiceResult raises a Lua error)"));
            return;
        }
        self.core.dice_results = results.iter().copied().collect();
    }

    // The declared code. The forms without a filter or with a type are the opcodes [TYPE_MONSTER | TYPE_SPELL |
    // TYPE_TRAP, OPCODE_ISTYPE] and [type, OPCODE_ISTYPE].
    pub fn announce_card(&mut self, player: u8, opcodes: &[u64]) -> Script<u32> {
        let mut stack_size = 0;
        for &opcode in opcodes {
            stack_size += match opcode {
                OPCODE_ADD | OPCODE_SUB | OPCODE_MUL | OPCODE_DIV | OPCODE_AND | OPCODE_OR => -1,
                OPCODE_BAND | OPCODE_BOR | OPCODE_BXOR | OPCODE_LSHIFT | OPCODE_RSHIFT => -1,
                OPCODE_NEG | OPCODE_NOT | OPCODE_BNOT | OPCODE_ALLOW_ALIASES | OPCODE_ALLOW_TOKENS => 0,
                OPCODE_ISCODE | OPCODE_ISSETCARD | OPCODE_ISTYPE | OPCODE_ISRACE | OPCODE_ISATTRIBUTE => 0,
                _ => 1,
            };
            if stack_size <= 0 {
                break;
            }
        }
        let has_opcodes = opcodes
            .iter()
            .any(|&opcode| opcode != OPCODE_ALLOW_ALIASES && opcode != OPCODE_ALLOW_TOKENS);
        if stack_size != 1 && has_opcodes {
            self.reach_unported(concat!(file!(), ":", line!(), " (Duel.AnnounceCard raises a Lua error)"));
        }
        self.core.select_options = opcodes.iter().copied().collect();
        if !has_opcodes {
            self.core
                .select_options
                .extend([u64::from(TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP), OPCODE_ISTYPE]);
        }
        self.emplace_process(AnnounceCard {
            playerid: player,
            ..AnnounceCard::default()
        });
        self.yield_script()?;
        Ok(self.returns.at_i32(0) as u32)
    }

    pub fn check_event(&self, code: u32) -> bool {
        self.field_check_event(code).is_some()
    }

    // Duel.CheckEvent with get_info: the raised event, whose fields are the values Lua returns after the boolean.
    pub fn check_event_full(&self, code: u32) -> Option<Event> {
        self.field_check_event(code)
    }

    pub fn get_attacker(&self) -> Option<CardId> {
        self.core.attacker
    }

    // A player but 0 or 1 returns nothing in Lua; the callers pass the duel's players.
    pub fn get_battle_damage(&self, player: u8) -> i32 {
        self.core.battle_damage[player]
    }

    pub fn is_attack_cost_paid(&self) -> u8 {
        self.core.attack_cost_paid
    }

    pub fn attack_cost_paid(&mut self) {
        self.attack_cost_paid_full(1);
    }

    pub fn attack_cost_paid_full(&mut self, paid: u8) {
        self.core.attack_cost_paid = paid;
    }

    pub fn chain_attack(&mut self) {
        let Some(attacker) = self.core.attacker else { return };
        if !self.is_affect_by_effect(attacker, self.core.reason_effect.get()) {
            return;
        }
        self.core.chain_attack = true;
        self.core.chain_attacker_id = self.cards[attacker].fieldid;
    }

    pub fn get_attack_target(&self) -> Option<CardId> {
        self.core.attack_target
    }

    pub fn calculate_damage(&mut self, attacker: CardId, attack_target: Option<CardId>) -> Script<()> {
        self.calculate_damage_full(attacker, attack_target, false)
    }

    pub fn calculate_damage_full(&mut self, attacker: CardId, attack_target: Option<CardId>, new_attack: bool) -> Script<()> {
        if Some(attacker) == attack_target {
            return Ok(());
        }
        self.emplace_process(crate::processor::DamageStep {
            attacker: Some(attacker),
            attack_target,
            new_attack,
            ..Default::default()
        });
        self.yield_script()
    }

    // The AttackDisable unit in one.
    pub fn negate_attack(&mut self) -> Script<bool> {
        let reason_effect = self.core.reason_effect.get();
        let Some(attacker) = self.core.attacker.filter(|&attacker| {
            self.core.effect_damage_step == 0
                && self.cards[attacker].fieldid_r == self.core.pre_field[0]
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
        self.field_raise_event(
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
