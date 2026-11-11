// Millennium Revelation
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, SZONE};

const ID: u32 = 41044418;

pub const LISTED_NAMES: &[u32] = &[CARD_MONSTER_REBORN, CARD_RA];

// The Winged Dragon of Ra in the hand to discard and in the GY to revive, Monster Reborn in the Deck and hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (CARD_RA, HAND),
    (CARD_RA, GRAVE),
    (CARD_RA, GRAVE),
    (CARD_MONSTER_REBORN, DECK),
    (CARD_MONSTER_REBORN, HAND),
    (CARD_MONSTER_REBORN, HAND),
    (CARD_MONSTER_REBORN, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(CARD_RA),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn thcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DIVINE) && c.is_able_to_grave_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, thcfilter, LOCATION_HAND)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MONSTER_REBORN) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, ID);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_operation(duel, regop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_count_limit(duel, 1, 0);
    e3.set_operation(duel, gyop);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn regfilter(c: CardId, duel: &Duel, tp: u8, re: Option<EffectId>) -> bool {
    c.is_faceup(duel)
        && c.is_original_code(duel, CARD_RA)
        && c.is_controler(duel, tp)
        && re.is_some()
        && c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + SUMMON_WITH_MONSTER_REBORN])
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, re) = (a.tp, a.re);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| regfilter(c, duel, tp, re), None);
    for tc in g {
        tc.register_flag_effect_full(
            duel,
            ID,
            RESETS_STANDARD_PHASE_END,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 3),
        );
    }
    Ok(())
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_original_code(duel, CARD_RA) && c.has_flag_effect(duel, ID)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(gyfilter, tp, LOCATION_MZONE, 0, None);
    if !g.is_empty() {
        duel.send_to_grave_full(g, REASON_RULE, PLAYER_NONE, tp)?;
    }
    Ok(())
}
