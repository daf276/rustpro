// Orichalcos Sword of Sealing
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 847217;

struct Procedure;

impl EquipProcedure for Procedure {}

// Umi in the hand for the Field Zone, Sangan on the field to equip and receive the effect, and an opponent's monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (22702055, HAND),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
    (5318639, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesGranted(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_DISABLE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_FZONE, 0) > 0);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = aux::faceup_filter(CardId::is_effect_monster);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, tp)
            && chkc.is_effect_monster(duel)
            && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(&filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, &filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_effect_monster(duel) && tc.is_faceup(duel) {
        tc.register_flag_effect_full(
            duel,
            ID,
            RESETS_STANDARD_PHASE_END,
            EFFECT_FLAG_CLIENT_HINT,
            2,
            0,
            aux::stringid(ID, 1),
        );
        let e1 = duel.create_effect(tc);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_category(duel, CATEGORY_DESTROY);
        e1.set_type(duel, EFFECT_TYPE_QUICK_O);
        e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_count_limit(duel, 1, 0);
        e1.set_cost(duel, descost);
        e1.set_target(duel, destg);
        e1.set_operation(duel, desop);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
