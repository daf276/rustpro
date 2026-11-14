// Drake Shark
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 81096431;

pub const LISTED_SERIES: &[u16] = &[SET_SHARK_DRAKE];

// In the Deck with Abyss Shark and WATER monsters to add it, and Number 32: Shark Drake with Xyz Import to attach it from the opponent's
// side, with Spells/Traps on the field to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (61496006, HAND),
    (61496006, HAND),
    (23771716, MZONE),
    (23771716, MZONE),
    (65676461, MZONE),
    (14602126, HAND),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (14602126, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesGranted(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_DRAW));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DOUBLE_XYZ_MATERIAL);
    e2.set_value(duel, 1);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_xyz_material_operation(duel, |duel, _, xyz, _, minxyzct| {
        minxyzct >= 3 && xyz.is_attribute(duel, ATTRIBUTE_WATER)
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_XMATERIAL + EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_set_card(duel, &[SET_SHARK_DRAKE]));
    e3.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e3.set_target(duel, attachtg);
    e3.set_operation(duel, attachop);
    c.register_effect(duel, e3);
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_spell_trap(duel) && (c.is_controler(duel, tp) || c.is_able_to_change_controler(duel))
}

// The Lua's target takes no chkc, so its chkc check reads an undefined global.
fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ)
            && duel.is_existing_target(|c, duel| attachfilter(c, duel, tp), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    duel.select_target(
        tp,
        |c, duel| attachfilter(c, duel, tp),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        1,
        None,
    )?;
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, a.e) && tc.is_relate_to_effect(duel, a.e) && c.is_type(duel, TYPE_XYZ) {
        duel.overlay(c, tc)?;
    }
    Ok(())
}
