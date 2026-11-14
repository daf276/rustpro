// Abyss Actor - Trendy Understudy
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, PZONE};

const ID: u32 = 39024589;

pub const LISTED_SERIES: &[u16] = &[SET_ABYSS_ACTOR];
// On the field with Abyss Actors in both Pendulum Zones (Extras) and the hand to Special Summon, and to Pendulum Summon for the search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, PZONE),
    (88412339, PZONE),
    (88412339, HAND),
    (88412339, HAND),
    (88412339, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(|c, duel| c.is_set_card(duel, &[SET_ABYSS_ACTOR]), a.tp, LOCATION_PZONE, 0, 2, None)
    });
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_pendulum_summoned(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    let lv = c.get_level(duel);
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_ABYSS_ACTOR])
        && (lv == 1 || lv == 8)
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_EXTRA, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    stock::search(duel, a, thfilter, LOCATION_EXTRA)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    if c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) == 0 {
        return false;
    }
    let lv = c.get_level(duel);
    (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_set_card(duel, &[SET_ABYSS_ACTOR])
        && (lv == 1 || lv == 8)
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut loc = LOCATION_EXTRA;
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if c.get_sequence(duel) < 5 {
        ft += 1;
    }
    if ft > 0 {
        loc |= LOCATION_HAND;
    }
    if chk == 0 {
        return Ok(loc != 0 && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, loc, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, loc as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut loc = LOCATION_EXTRA;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        loc |= LOCATION_HAND;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, loc, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
