// Void Seer
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 8437145;

// Infernoids on the field to protect, with a copy in the GY against the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (14799437, MZONE),
    (23440231, MZONE),
    (51316684, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// The replacement is a continuous effect with no chain link: its prompt is checked.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(Some(c) == e.get_label_object_group(duel).and_then(|g| g.get_first()))),
    );
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_INFERNOID])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_controler(duel, tp)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3110);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(
            duel,
            Value::Effect(|duel, e, re| e.get_owner_player(duel) != re.get_owner_player(duel)),
        );
        e1.set_owner_player(duel, tp);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_set_card(duel, &[SET_INFERNOID])
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && eg.is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    let g = eg.filter(duel, |c, duel| repfilter(c, duel, tp), None);
    let selected = if g.len() == 1 {
        g
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
        g.select(duel, tp, 1, 1, None)?
    };
    e.set_label_object_group(duel, selected.get_first().map(CardSet::from));
    duel.remove(c, POS_FACEUP, REASON_EFFECT)?;
    Ok(true)
}
