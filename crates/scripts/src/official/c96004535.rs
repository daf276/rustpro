// Unbreakable Xyz Barrier
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 96004535;

// Tornado Dragon on the field, and Number 39: Utopia in the GY to attach.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (84013237, GRAVE), (6983839, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.negate_activation(a.ev as u8)
        && duel.check_remove_overlay_card(tp, 1, 0, 2, REASON_EFFECT)
        && duel.is_existing_matching_card(|_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        if duel.remove_overlay_card(tp, 1, 0, 2, 2, REASON_EFFECT)? == 2 {
            let c = a.e.get_handler(duel);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let dg = duel.select_matching_card(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
            if dg.is_empty() {
                return Ok(());
            }
            duel.hint_selection(dg);
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn matfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_set_card(duel, &[SET_UTOPIC])
        && c.is_type(duel, TYPE_XYZ)
        && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_XYZ)
        && duel.is_existing_matching_card(|mc, duel| matfilter(mc, duel, tp, c), tp, LOCATION_GRAVE, 0, 1, None)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| xyzfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| xyzfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let g = duel.select_matching_card(tp, |c, duel| matfilter(c, duel, tp, tc), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.overlay(tc, g)?;
        }
    }
    Ok(())
}
