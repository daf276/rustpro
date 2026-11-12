// Epurrely Beauty
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 98049934;

pub const LISTED_NAMES: &[u32] = &[29599813];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

// On the field with Purrely Pretty Memory and Purrely Happy Memory in the hand to activate and attach, and Sangan to negate and turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (29599813, HAND),
    (29599813, HAND),
    (82105704, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_POSITION);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 3, 0);
    e3.set_condition(duel, qpovcon);
    e3.set_target(duel, qpovtg);
    e3.set_operation(duel, qpovop);
    c.register_effect(duel, e3);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let overlay = a.e.get_handler(duel).get_overlay_group(duel);
    a.e.is_has_type(duel, EFFECT_TYPE_QUICK_O) == overlay.is_exists(duel, |c, duel| c.is_code(duel, 29599813), 1, None)
}

fn negfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && negfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(negfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, negfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) {
        let c = a.e.get_handler(duel);
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn qpovcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.rp == 1 - a.tp || !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return false;
    }
    let rc = re.get_handler(duel);
    rc.is_set_card(duel, &[SET_PURRELY])
        && rc.is_quick_play_spell(duel)
        && rc.is_on_field(duel)
        && rc.is_can_be_xyz_material(duel, Some(a.e.get_handler(duel)), a.tp, REASON_EFFECT)
}

fn qpovtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, 1 - a.tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn qpovop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && rc.is_relate_to_effect(duel, re)
        && !c.is_immune_to_effect(duel, e)
        && !rc.is_immune_to_effect(duel, e)
        && rc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, rc)?;
        if !c.get_overlay_group(duel).is_contains(rc) {
            return Ok(());
        }
        rc.cancel_to_grave(duel);
        if duel.is_existing_matching_card(CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let g = duel.select_matching_card(tp, CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if g.is_empty() {
                return Ok(());
            }
            duel.hint_selection_full(g, true);
            duel.break_effect()?;
            duel.change_position_full(g, [POS_FACEUP_DEFENSE, 0, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK], false)?;
        }
    }
    Ok(())
}
