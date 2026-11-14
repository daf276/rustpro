// Solfachord Primoa
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, PZONE};

const ID: u32 = 11688916;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_SOLFACHORD];
// DoSolfachord Cutia and FaSolfachord Fancia to add and as materials for GranSolfachord Musecia, and Solfachord Happiness in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (55226153, DECK),
    (54100561, DECK),
    (54100561, MZONE),
    (54100561, MZONE),
    (37972500, EXTRA),
    (37972500, EXTRA),
    (93481594, GRAVE),
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
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| pendthconfilter(c, duel, a.tp), 1, None))
    });
    e1.set_target(duel, pendthtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_DISEFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, immcon);
    e2.set_value(duel, Value::Chain(immval));
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 1));
    e3a.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_count_limit(duel, 1, ID);
    e3a.set_target(duel, deckthtg);
    e3a.set_operation(duel, deckthop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| gyedthconfilter(c, duel, a.tp), 1, None))
    });
    e4.set_target(duel, gyedthtg);
    e4.set_operation(duel, gyedthop);
    c.register_effect(duel, e4);
}

fn pendthconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_pendulum_summoned(duel)
}

fn pendthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_able_to_hand(duel)
}

fn pendthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && pendthfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(pendthfilter, tp, LOCATION_PZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, pendthfilter, tp, LOCATION_PZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn immcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(
        |c, duel| c.is_set_card(duel, &[SET_SOLFACHORD]),
        a.e.get_handler_player(duel),
        LOCATION_PZONE,
        0,
        None,
    );
    let mut cards = g.iter();
    let (Some(first), Some(second)) = (cards.next(), cards.next()) else {
        return false;
    };
    (first.get_scale(duel) + second.get_scale(duel)) % 2 == 1
}

fn immval(duel: &Duel, e: EffectId, ct: u8) -> bool {
    chain::is_triggering_player(duel, ct, e.get_handler_player(duel)) && chain::is_setcode(duel, ct, &[SET_SOLFACHORD])
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn deckthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, deckthfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn deckthop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, deckthfilter, LOCATION_DECK)
}

fn gyedthconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_link_summoned(duel) && c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_summon_player(duel, tp)
}

fn gyedthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn gyedthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, gyedthfilter, LOCATION_GRAVE | LOCATION_EXTRA, CATEGORY_TOHAND)
}

fn gyedthop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(gyedthfilter), LOCATION_GRAVE | LOCATION_EXTRA)
}
