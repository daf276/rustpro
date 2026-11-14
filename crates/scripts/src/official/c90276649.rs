// Couplet the Melodious Songstress
use super::c64881644::add_pendulum_placement;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, PZONE};

const ID: u32 = 90276649;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS];

const REFRAIN: u32 = 64881644;
const CONCERTO: u32 = 31458630;

// In the Pendulum Zone and the Deck, added by Refrain from the hand; Melodious Concerto in the Deck to add, Bacha and
// Polymerization for the Fusion Summon the Extra Deck copy reacts to, Aria and Elegy in the hand and GY to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, DECK),
    (ID, DECK),
    (REFRAIN, HAND),
    (REFRAIN, HAND),
    (CONCERTO, DECK),
    (CONCERTO, DECK),
    (56208713, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (40502912, MZONE),
    (40502912, GRAVE),
    (79514956, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtp, _, _, _| {
        !c.is_attribute(duel, ATTRIBUTE_LIGHT) && sumtp & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, |duel, a, chk, _| {
        stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
    });
    e2.set_operation(duel, |duel, a| stock::search(duel, a, thfilter, LOCATION_DECK));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_HAND);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_RULE));
    e3.set_cost(duel, |duel, a, chk| Ok(chk == 0 && !a.e.get_handler(duel).is_public(duel)));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    add_pendulum_placement(duel, c, ID);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, 0, None);
    g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_MELODIOUS]), None) == g.len()
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
