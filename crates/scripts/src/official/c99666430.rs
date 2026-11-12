// Jinzo - Layered
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 99666430;

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters (Frostosaurus, Boulder Tortoise), with the opponent's monsters to take
// and Continuous Traps (Limit Reverse) on the field for the second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (9540040, MZONE),
    (6631034, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (27551, SZONE),
    (27551, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<6, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, ctrltg);
    e1.set_operation(duel, ctrlop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_trap),
            a.tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            None,
        )
    });
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn ctrltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = aux::faceup_filter(CardId::is_controler_can_be_changed);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_faceup(duel)
            && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(&filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, &filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn ctrlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        let reason_player = duel.get_reason_player();
        if duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, reason_player)? > 0 {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3302);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END | RESET_CONTROL);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_description(duel, 3206);
            e2.set_code(duel, EFFECT_CANNOT_ATTACK);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn tribfilter(c: CardId, duel: &Duel) -> bool {
    let exception = c.get_equip_group(duel).union(c.into());
    c.is_releasable_by_effect(duel)
        && duel.is_existing_matching_card(CardId::is_faceup, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(exception))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_full(tp, tribfilter, 1, ReleaseOptions::new(tp)));
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rg = duel.select_release_group_full(tp, tribfilter, (1, 1), false, ReleaseOptions::new(tp))?;
    if !rg.is_empty() && duel.release(rg, REASON_EFFECT)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = duel.select_matching_card(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        duel.hint_selection(dg);
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
