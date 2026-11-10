// Chronomaly Vimana
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 2609443;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

// On the field with Xyz materials, and Dark Rebellion Xyz Dragon in the Graveyard to attach; Raigeki sends the opponent's Sangan
// to the Graveyard for its effect to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (16195942, GRAVE),
    (16195942, GRAVE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMING_END_PHASE);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_CHRONOMALY]) || c.is_type(duel, TYPE_XYZ))
        && c.is_monster(duel)
        && c.get_attack(duel) > 0
        && !c.is_forbidden(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
                && duel.is_existing_target(gyfilter, tp, LOCATION_GRAVE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g1 = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g2 = duel.select_target(tp, gyfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g1, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g2, 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let op_cards = |duel: &Duel, category| {
        duel.get_operation_info(0, category)
            .and_then(|info| info.op_cards)
            .unwrap_or_default()
    };
    let (g1, g2) = (op_cards(duel, CATEGORY_ATKCHANGE), op_cards(duel, CATEGORY_LEAVE_GRAVE));
    let c = e.get_handler(duel);
    let (Some(tc1), Some(tc2)) = (g1.get_first(), g2.get_first()) else {
        return Ok(());
    };
    if tc1.is_relate_to_effect(duel, e) && tc1.is_faceup(duel) && !tc1.is_immune_to_effect(duel, e) && tc2.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, tc2.get_attack(duel).div_euclid(2));
        tc1.register_effect(duel, e1);
        if tc1.is_has_effect(duel, EFFECT_REVERSE_UPDATE).is_none() && c.is_relate_to_effect(duel, e) {
            duel.break_effect()?;
            duel.overlay(c, tc2)?;
        }
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
