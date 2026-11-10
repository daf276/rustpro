// Secure Gardna
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 2220237;

// In the Extra Deck, Link Summoned with Cyberse Link Monsters (Linkuriboh and Salamangreat Almiraj) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (ID, EXTRA), (41999284, MZONE), (60303245, MZONE)];
// The damage prevention shows no message of its own.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp) && c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    // The code argument of SetUniqueOnField is the filter "has this code".
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_value(duel, Value::ChangeDamage(damval2));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e4.set_condition(duel, |duel, a| !a.e.get_handler(duel).has_flag_effect(duel, ID));
    c.register_effect(duel, e4);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval1));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    e3.set_target_range(duel, 1, 0);
    duel.register_effect(e3, tp);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}

fn damval1(_: &mut Duel, _: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT != 0 { 0 } else { val }
}

fn damval2(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    let c = e.get_handler(duel);
    if r & (REASON_BATTLE | REASON_EFFECT) != 0 && !c.has_flag_effect(duel, ID) {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        duel.hint(HINT_CARD, 0, u64::from(ID));
        return 0;
    }
    val
}
