// Artillery Catapult Turtle
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 7913375;

// Copies on the field, with Gaia The Fierce Knight and Level 5 Dragons (Curse of Dragon, Parrot Dragon) to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (6368038, DECK),
    (28279543, DECK),
    (62762898, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c: CardId, duel: &Duel| cfilter(c, duel, ft, tp);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel, ft: i32, tp: u8) -> bool {
    ft > 0 || (c.is_controler(duel, tp) && c.get_sequence(duel) < 5)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_GAIA_THE_FIERCE_KNIGHT]) || (c.is_race(duel, RACE_DRAGON) && c.is_level(duel, &[5])))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_HAND,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_DECK | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
