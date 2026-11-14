// D/D Scale Surveyor
use crate::prelude::*;
use crate::testing::{Check, HAND, PZONE};

const ID: u32 = 42382265;

pub const LISTED_SERIES: &[u16] = &[SET_DD];
// In a Pendulum Zone with D/D Orthros in the other, and copies in the hand to Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (72181263, PZONE), (ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, scaletg);
    e1.set_operation(duel, scaleop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(ddpendmonfilter, a.tp, LOCATION_ONFIELD, 0, 1, None)
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, lvtg);
    e3.set_operation(duel, lvop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_TOHAND);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_count_limit(duel, 1, (ID, 3));
    e5.set_target(duel, thtg);
    e5.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e5);
    let e6 = e5.clone_effect(duel);
    e6.set_code(duel, EVENT_TO_DECK);
    e6.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)
    });
    c.register_effect(duel, e6);
}

fn scale_not_zero(c: CardId, duel: &Duel) -> bool {
    !c.is_scale(duel, &[0])
}

fn scaletg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && !chkc.is_scale(duel, &[0]));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(scale_not_zero, tp, LOCATION_PZONE, LOCATION_PZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, scale_not_zero, tp, LOCATION_PZONE, LOCATION_PZONE, 1, 2, None)?;
    Ok(false)
}

fn scaleop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tg) = duel.get_target_cards(a.e).filter(|tg| !tg.is_empty()) else {
        return Ok(());
    };
    let c = a.e.get_handler(duel);
    for tc in tg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_LSCALE);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CHANGE_RSCALE);
        e2.set_value(duel, 0);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn ddpendmonfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_original_type(duel, TYPE_PENDULUM) && c.is_monster_card(duel) && c.is_faceup(duel)
}

// The Lua passes an undefined global `c` as the target, so it is nil.
fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(!a.e.get_handler(duel).is_level(duel, &[4]));
    }
    duel.set_operation_info(0, CATEGORY_LVCHANGE, None, 1, a.tp, 4);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 4);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ddpendmonfilter(c, duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
