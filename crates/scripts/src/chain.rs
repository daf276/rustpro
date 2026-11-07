// Port of chain.lua: chain data, the Chain boolean checks and the effect card properties scripts reach.
use engine::{CardId, CardSet, Duel, EffectId};

use crate::constant::*;
use crate::proc_workaround::CardWorkaround;

// Chain.IsTriggeringLocation: the real or the symbolic triggering location of the chain link has common bits.
pub fn is_triggering_location(duel: &Duel, ch: u8, location: u32) -> bool {
    duel.get_chain_info::<u32>(ch, CHAININFO_TRIGGERING_LOCATION) & location != 0
        || duel.get_chain_info::<u32>(ch, CHAININFO_TRIGGERING_LOCATION_SYMBOLIC) & location != 0
}

// Chain.IsTriggeringControler, IsTriggeringType and IsTriggeringSetcode, false where they return nil (no such link).
pub fn is_triggering_controler(duel: &Duel, ch: u8, player: u8) -> bool {
    duel.get_chain_info::<Option<u8>>(ch, CHAININFO_TRIGGERING_CONTROLER) == Some(player)
}

pub fn is_triggering_type(duel: &Duel, ch: u8, card_type: u32) -> bool {
    duel.get_chain_info::<Option<u32>>(ch, CHAININFO_TRIGGERING_TYPE)
        .is_some_and(|triggering| triggering & card_type > 0)
}

pub fn is_triggering_setcode(duel: &Duel, ch: u8, setcodes: &[u16]) -> bool {
    duel.get_chain_info::<Option<[u16; 5]>>(ch, CHAININFO_TRIGGERING_SETCODES)
        .is_some_and(|triggering| {
            triggering
                .iter()
                .any(|&sa| setcodes.iter().any(|&sb| (sa & 0xfff) == (sb & 0xfff) && (sa & sb) == sb))
        })
}

// Chain.IsSetcode(ch, setcodes), false where it returns nil. The resolving properties are recorded only in puzzles, so a card
// that is no longer related to its effect or face-up falls back to the triggering properties.
pub fn is_setcode(duel: &Duel, ch: u8, setcodes: &[u16]) -> bool {
    if duel.get_current_chain() == 0 {
        return false;
    }
    let Some(eff) = duel.get_chain_info::<Option<EffectId>>(ch, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    if eff.is_has_property(duel, EFFECT_FLAG_FIELD_ONLY) {
        return properties_card(duel, eff).is_some_and(|card| card.is_set_card(duel, setcodes));
    }
    let ec = eff.get_handler(duel);
    if ec.is_relate_to_effect(duel, eff) && ec.is_faceup(duel) {
        return ec.is_set_card(duel, setcodes);
    }
    is_triggering_setcode(duel, ch, setcodes)
}

// e:GetChainData() of the current chain link, keyed by its chain id and the effect; a `slot` per field of the table.
fn chain_data_key(duel: &Duel, e: EffectId, slot: u8) -> u64 {
    let chain_id = u64::from(duel.get_chain_info::<u32>(duel.get_current_chain(), CHAININFO_CHAIN_ID));
    (chain_id << 32) | ((e.index() as u64) << 8) | u64::from(slot) | (1 << 63)
}

pub trait EffectChain: Copy {
    fn get_chain_data(self, duel: &Duel, slot: u8) -> Option<i64>;
    fn set_chain_data(self, duel: &mut Duel, slot: u8, value: i64);
    fn get_chain_data_group(self, duel: &Duel, slot: u8) -> Option<CardSet>;
    fn set_chain_data_group(self, duel: &mut Duel, slot: u8, group: CardSet);
    fn is_card_setcode(self, duel: &Duel, setcode: u16) -> bool;
    fn is_card_attribute(self, duel: &Duel, attribute: u32) -> bool;
}

impl EffectChain for EffectId {
    // Lua's nil field is None; values are kept offset by one so that 0 stays unset.
    fn get_chain_data(self, duel: &Duel, slot: u8) -> Option<i64> {
        let value = duel.get_script_state(chain_data_key(duel, self, slot));
        (value != 0).then(|| value - 1)
    }

    fn set_chain_data(self, duel: &mut Duel, slot: u8, value: i64) {
        let key = chain_data_key(duel, self, slot);
        duel.set_script_state(key, value + 1);
    }

    fn get_chain_data_group(self, duel: &Duel, slot: u8) -> Option<CardSet> {
        duel.get_script_group(chain_data_key(duel, self, slot))
    }

    fn set_chain_data_group(self, duel: &mut Duel, slot: u8, group: CardSet) {
        let key = chain_data_key(duel, self, slot);
        duel.set_script_group(key, Some(group));
    }

    // Effect.IsCardSetcode.
    fn is_card_setcode(self, duel: &Duel, setcode: u16) -> bool {
        properties_card(duel, self).is_some_and(|card| card.is_set_card(duel, &[setcode]))
    }

    // Effect.IsCardAttribute.
    fn is_card_attribute(self, duel: &Duel, attribute: u32) -> bool {
        properties_card(duel, self).is_some_and(|card| card.is_attribute(duel, attribute))
    }
}

// The card whose properties chain.lua's effect_card_prop reads for `e`, None where Lua reads nil. The handler also stands
// for its resolving properties (recorded only in puzzles) and triggering ones (the same card data).
fn properties_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    if e.is_has_property(duel, EFFECT_FLAG_FIELD_ONLY) {
        let registering = duel.get_script_group(registering_key(e)).and_then(|card| card.get_first());
        return registering.filter(|_| e.get_owner(duel) != CardId::default());
    }
    let triggering = duel.get_current_chain() > 0 && duel.get_chain_info::<Option<EffectId>>(0, CHAININFO_TRIGGERING_EFFECT) == Some(e);
    Some(if e.is_activated(duel) && triggering {
        e.get_handler(duel)
    } else {
        e.get_owner(duel)
    })
}

fn registering_key(e: EffectId) -> u64 {
    (1 << 61) | e.index() as u64
}

pub trait DuelChain {
    fn register_effect(&mut self, e: EffectId, player: u8);
}

impl DuelChain for Duel {
    // chain.lua's override records the properties of the registering effect's card; only its archetypes, which come
    // from the card's data, are ever read. A new effect in a reused slot starts unrecorded.
    fn register_effect(&mut self, e: EffectId, player: u8) {
        let recorded = match self.get_reason_effect() {
            Some(re) if e.get_owner(self) != CardId::default() => properties_card(self, re),
            _ => None,
        };
        self.set_script_group(registering_key(e), recorded.map(CardSet::from));
        self.register_effect_core(e, player);
    }
}
