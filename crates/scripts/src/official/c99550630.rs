// Sky Striker Maneuver - Afterburners!
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 99550630;

// The opponent's monsters and Spells to destroy, three Spells in the Graveyard for the second destruction.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (75652080, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, super::c25733157::condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.destroy(tc, REASON_EFFECT)? > 0 {
        let dg = duel.get_matching_group(
            CardId::is_spell_trap,
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            Some(e.get_handler(duel).into()),
        );
        if !dg.is_empty()
            && duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3
            && duel.select_yes_no(tp, aux::stringid(ID, 0))?
        {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = dg.select(duel, tp, 1, 1, None)?;
            duel.hint_selection(sg);
            duel.destroy(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
