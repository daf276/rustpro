// Armed Dragon LV7
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 73879377;
const ARMED_DRAGON_LV5: u32 = 46384672;

pub const LISTED_NAMES: &[u32] = &[ARMED_DRAGON_LV5];
// On the field with weak monsters (Tomozaurus, Anthrosaurus) on the opponent's side to destroy, and monsters in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (46457856, MZONE | OPPONENT),
    (89904598, MZONE | OPPONENT),
    (46457856, HAND),
    (89904598, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel).max(0);
    c.is_monster(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(|c, duel| dfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None)
}

fn dfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) <= atk
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let atk = g.get_first().map_or(0, |c| c.get_attack(duel).max(0));
    e.set_label(duel, i64::from(atk));
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (tp, atk) = (a.tp, a.e.get_label(duel) as i32);
    let g = duel.get_matching_group(|c, duel| dfilter(c, duel, atk), tp, 0, LOCATION_MZONE, None);
    let count = g.len() as u8;
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), count, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let atk = a.e.get_label(duel) as i32;
    let g = duel.get_matching_group(|c, duel| dfilter(c, duel, atk), a.tp, 0, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
