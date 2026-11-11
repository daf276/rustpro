// Decayed Commander
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 10209545;
const ZOMBIE_TIGER: u32 = 47693640;

pub const LISTED_NAMES: &[u32] = &[ZOMBIE_TIGER];
// Summoned from the hand with Zombie Tiger in it, and on the field to attack directly.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (ZOMBIE_TIGER, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, hdcon);
    e2.set_target(duel, hdtg);
    e2.set_operation(duel, hdop);
    c.register_effect(duel, e2);
}

pub fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ZOMBIE_TIGER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn hdcon(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && duel.get_attack_target().is_none()
}

pub fn hdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, 1);
    Ok(false)
}

pub fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, 1 - tp, 1);
    duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    Ok(())
}
