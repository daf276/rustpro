// Superheavy Samurai Monk Big Benkei
use super::c82112494::{gycon, pencon, penop, pentg};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, PZONE};

const ID: u32 = 19510093;
const BIG_BENKEI: u32 = 3117804;

pub const LISTED_NAMES: &[u32] = &[BIG_BENKEI];
pub const LISTED_SERIES: &[u16] = &[SET_SUPERHEAVY_SAMURAI, SET_SUPERHEAVY_SAMURAI_SOUL];
// Superheavy Samurai Drum (Level 1 Tuner) on the field for the search and as a material with Naturia Leodrake (Level 9),
// Big Benkei in the Deck to send, and Soulpiercer and Soulgaia Booster in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, MZONE),
    (87535691, MZONE),
    (87535691, MZONE),
    (16527176, EXTRA),
    (16527176, EXTRA),
    (16527176, EXTRA),
    (BIG_BENKEI, DECK),
    (BIG_BENKEI, DECK),
    (90361010, DECK),
    (56727340, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI]),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, gycon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, pencon);
    e3.set_target(duel, pentg);
    e3.set_operation(duel, penop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI_SOUL]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, BIG_BENKEI) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, Some(c.into()))
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, Some(c.into()))?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
