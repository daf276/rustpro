// Beast of Talwar - The Sword Summit
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 20011655;

// In the hand, Special Summoned with no monsters, with Axe of Despair (an Equip Spell) in each hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (40619825, HAND), (40619825, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_value(
        duel,
        Value::Reason(|duel, e, _, r, rp| r & REASON_EFFECT > 0 && rp == 1 - e.get_handler_player(duel)),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_SET_AVAILABLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, e, c| {
        c != e.get_handler(duel) && c.is_monster(duel) && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
    });
    e3.set_value(duel, Value::EffectPlayer(tgval));
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn tgval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    if !(re.is_spell_effect(duel) && re.is_active_type(duel, TYPE_EQUIP)) {
        return false;
    }
    let typ: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_TYPE);
    let eff: Option<EffectId> = duel.get_chain_info(0, CHAININFO_TRIGGERING_EFFECT);
    if eff == Some(re) {
        typ & (TYPE_SPELL | TYPE_EQUIP) == TYPE_SPELL | TYPE_EQUIP
    } else {
        re.get_handler(duel).is_equip_spell(duel)
    }
}
