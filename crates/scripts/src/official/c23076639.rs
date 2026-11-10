// Gazelle the King of Mythical Claws
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 23076639;

pub const LISTED_NAMES: &[u32] = &[CARD_CHIMERA_FUSION];

// Morinphen (Level 5 Fiend) and Jongleur-Ghoul Illusionist (Illusion), Normal Monsters, to search. Chimera the King of Phantom Beasts
// is Fusion Summoned by Polymerization with Gazelle (Beast) and Morinphen (Fiend) as materials.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (24094653, HAND),
    (1769875, EXTRA),
    (55784832, HAND),
    (55784832, DECK),
    (55784832, DECK),
    (47894537, DECK),
    (47894537, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg1);
    e1.set_operation(duel, thop1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r & REASON_FUSION == REASON_FUSION
    });
    e3.set_target(duel, thtg2);
    e3.set_operation(duel, thop2);
    c.register_effect(duel, e3);
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && (c.is_level(duel, &[5]) && c.is_race(duel, RACE_FIEND) || c.is_code(duel, CARD_CHIMERA_FUSION))
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && c.is_race(duel, RACE_ILLUSION)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, filter: fn(CardId, &Duel) -> bool) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args, filter: fn(CardId, &Duel) -> bool) -> Script<()> {
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(a.tp, filter, a.tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, g)?;
    }
    Ok(())
}

fn thtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    thtg(duel, a, chk, thfilter1)
}

fn thop1(duel: &mut Duel, a: &Args) -> Script<()> {
    thop(duel, a, thfilter1)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    thtg(duel, a, chk, thfilter2)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    thop(duel, a, thfilter2)
}
