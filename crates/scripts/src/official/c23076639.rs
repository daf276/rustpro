// Gazelle the King of Mythical Claws
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 23076639;

pub const LISTED_NAMES: &[u32] = &[CARD_CHIMERA_FUSION];

// Twice in hand, one as Fusion Material with Souls of the Forgotten (Fiend) for Chimera the King of Phantom Beasts by Polymerization,
// with Berfomet and Jongleur-Ghoul Illusionist (Illusion) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (24094653, HAND),
    (4920010, HAND),
    (1769875, EXTRA),
    (77207191, DECK),
    (77207191, DECK),
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
    e1.set_target(duel, |duel, a, chk, _| thtg(duel, a, chk, thfilter1));
    e1.set_operation(duel, |duel, a| thop(duel, a, thfilter1));
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
    e3.set_condition(duel, super::c1498449::matcon);
    e3.set_target(duel, |duel, a, chk, _| thtg(duel, a, chk, thfilter2));
    e3.set_operation(duel, |duel, a| thop(duel, a, thfilter2));
    c.register_effect(duel, e3);
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && (c.is_level(duel, &[5]) && c.is_race(duel, RACE_FIEND) || c.is_code(duel, CARD_CHIMERA_FUSION))
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && c.is_race(duel, RACE_ILLUSION)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, filter: fn(CardId, &Duel) -> bool) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args, filter: fn(CardId, &Duel) -> bool) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
