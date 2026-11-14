// Heart of the Blue-Eyes
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 54475145;

pub const LISTED_SERIES: &[u16] = &[SET_MILLENNIUM];
pub const LISTED_NAMES: &[u32] = &[CARD_MILLENNIUM_CROSS];

// In the hand to discard and in the Graveyard, with Millennium Ankh and its five limbs of the Forbidden One to activate, and the opponent
// Summoning Blue-Eyes White Dragon from the hand by Monster Reborn or the tribute of two monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_MILLENNIUM_CROSS, HAND),
    (CARD_MILLENNIUM_CROSS, DECK),
    (7902349, HAND),
    (8124921, HAND),
    (44519536, HAND),
    (70903634, HAND),
    (7902349, DECK),
    (83257450, EXTRA),
    (CARD_BLUEEYES_W_DRAGON, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (CARD_BLUEEYES_W_DRAGON, GRAVE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_CHANGE_CONTROL);
    e2.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit_full(duel, 1, (ID, 1), EFFECT_COUNT_CODE_DUEL);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_CHAIN_SOLVED);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && !duel.has_flag_effect(a.rp, ID)
        && re.get_handler(duel).is_code(duel, CARD_MILLENNIUM_CROSS)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
    {
        duel.register_flag_effect(a.rp, ID, 0, 0, 0);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MILLENNIUM_CROSS) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, 1 - tp)
        && (c.is_level_above(duel, 8) || c.is_attack_above(duel, 3000))
        && c.is_faceup(duel)
        && c.is_able_to_grave(duel)
        && c.is_location(duel, LOCATION_MZONE)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.has_flag_effect(tp, ID)
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, tp), 1, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| spconfilter(c, duel, tp), None);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(g)) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if !g.is_empty()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None)
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
