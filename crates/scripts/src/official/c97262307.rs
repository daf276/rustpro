// Ragnaraika Wicked Butterfly
use super::c42307760::RACES;
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 97262307;

// In hand with Insect Normal Monsters to summon, and in the Graveyard for Ragnaraika Mantis Monk to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (58192742, HAND),
    (58192742, HAND),
    (58192742, GRAVE),
    (70514456, MZONE),
    (89824842, DECK),
    (89824842, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND | LOCATION_GRAVE)
    });
    e2.set_target(duel, regtg);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACES) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND,
                0,
                1,
                Some(e.get_handler(duel).into()),
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn regtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 1);
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_operation(duel, drop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 0, ID.into());
    let g = duel.get_matching_group(aux::faceup_filter(|c, duel| c.is_race(duel, RACES)), tp, LOCATION_MZONE, 0, None);
    if !g.is_empty() {
        duel.draw(tp, g.get_class_count(duel, |c, duel| c.get_race(duel) as i64) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
