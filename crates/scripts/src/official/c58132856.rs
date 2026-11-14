// Toy Magician
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 58132856;

pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in hand to Set and face-down on the field to Flip Summon, Harpie's Feather Duster on the opponent's side to destroy the Set ones.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (18144506, HAND | OPPONENT),
    (18144506, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MONSTER_SSET);
    e1.set_value(duel, TYPE_SPELL as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_previous_location(duel, LOCATION_SZONE)
        && c.is_previous_position(duel, POS_FACEDOWN)
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_reason(duel, REASON_DESTROY)
        && a.rp != a.tp
    {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 0);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) > 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, ID)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let g = duel.get_matching_group(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), ct.min(g.len()) as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, ct as u8, ct as u8, None)?;
    duel.hint_selection(g);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
