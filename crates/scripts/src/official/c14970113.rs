// Zoodiac Hammerkong
use super::c11510448::{Procedure, atkval, defval};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 14970113;

pub const LISTED_SERIES: &[u16] = &[SET_ZOODIAC];
// In the Extra Deck, Xyz Summoned with Level 4 Normal Monsters (Prevent Rat, Kojikocy, Uraby) or over Zoodiac Boarbow on the field,
// and Zoodiac Boarbow in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (74393852, MZONE),
    (74393852, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<ID, 3, { Xyz::INFINITE_MATS }>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_value(duel, Value::Card(defval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, efftg);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, rmtg);
    e4.set_operation(duel, rmop);
    c.register_effect(duel, e4);
}

fn efftg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_ZOODIAC]) && c != e.get_handler(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_OPSELECTED, 1 - a.tp, a.e.get_description(duel));
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
    Ok(())
}
