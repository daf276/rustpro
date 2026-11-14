// Angelechy Brilliant Move
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 101402093;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];
// In the hand with Angelechy Problem to place Angelechy Destrier from the Extra Deck when it summons Angelechy Enlisted, and in the
// Graveyard with a second Problem to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (17782288, HAND),
    (17782288, HAND),
    (81797573, EXTRA),
    (55393975, EXTRA),
    (55393975, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, super::c17782288::tdtg);
    e1.set_operation(duel, |duel, a| super::c17782288::return_to_extra(duel, a, aux::stringid(ID, 2)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::self_banish, spcost]));
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_EXTRA>);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY])
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, Some(c)), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: Option<CardId>) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, mc.map(CardSet::from), c) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(
        duel,
        a,
        chk,
        |c, duel| spcostfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_ONFIELD,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
