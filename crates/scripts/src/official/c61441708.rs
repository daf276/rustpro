// Sacred Phoenix of Nephthys
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 61441708;

// One on the field, destroyed by the opponent's Dark Hole to return in the Standby Phase.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (53129443, HAND | OPPONENT), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_operation(duel, spr);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn spr(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if a.r & (REASON_DESTROY | REASON_EFFECT) != (REASON_DESTROY | REASON_EFFECT) {
        return Ok(());
    }
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN;
    if duel.is_turn_player(tp) && duel.is_phase(PHASE_STANDBY) {
        let turn = i64::from(duel.get_turn_count());
        e.set_label(duel, turn);
        c.register_flag_effect(duel, ID, reset, 0, 2);
    } else {
        e.set_label(duel, 0);
        c.register_flag_effect(duel, ID, reset, 0, 1);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel)
        .is_some_and(|e1| e1.get_label(duel) != i64::from(duel.get_turn_count()))
        && a.tp == duel.get_turn_player()
        && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(CardId::is_spell_trap, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_spell_trap, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
