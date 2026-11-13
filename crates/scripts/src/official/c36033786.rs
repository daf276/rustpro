// Cyberse Wizard
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 36033786;

// On the field with Elfnote Lucina on the opponent's side to switch to Defense Position.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(posfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, posfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.change_position(tc, POS_FACEUP_DEFENSE)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_value(
            duel,
            Value::Card(|duel, e, c| i32::from(e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c))),
        );
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_PIERCE);
        e2.set_target_range(duel, LOCATION_MZONE, 0);
        e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_CYBERSE));
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, tp);
    }
    Ok(())
}
