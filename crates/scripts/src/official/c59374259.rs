// Furtive Techniques Fur Hire, Fur All Your Ultimate Moves
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 59374259;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
// Set with Fur Hire monsters on the field and Sangan to destroy; a copy in the Graveyard returns when Mayhem Fur Hire revives
// Rafale (Level 8).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (ID, GRAVE),
    (93850652, MZONE),
    (47647354, MZONE),
    (26202165, MZONE | OPPONENT),
    (91405870, HAND),
    (38916526, GRAVE),
    (38916526, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2);
}

fn fur_hire_faceup(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(fur_hire_faceup, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(fur_hire_faceup, tp, LOCATION_MZONE, 0, None);
    let ct = g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tg = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct, Some(c.into()))?;
    let count = tg.map_or(0, |tg| tg.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, count, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, tg, count, tp, 0);
    Ok(false)
}

fn lv8filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 8) && c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_faceup(duel)
}

fn link3filter(c: CardId, duel: &Duel) -> bool {
    c.is_link_above(duel, 3) && c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_faceup(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    if duel.is_existing_matching_card(lv8filter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(link3filter, tp, LOCATION_MZONE, 0, 1, None)
        && tg.is_exists(duel, CardId::is_able_to_remove, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    } else {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(
        duel,
        |c, duel| (lv8filter(c, duel) || link3filter(c, duel)) && c.is_controler(duel, tp),
        1,
        None,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
