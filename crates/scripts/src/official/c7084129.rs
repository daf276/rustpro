// Magician's Rod
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 7084129;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];
// In the hand and Graveyard, with Dark Magical Circle in the Deck to add, Mystical Elf, a Spellcaster Normal Monster, to Tribute and
// Trap Hole set to activate on the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (47222536, DECK),
    (47222536, DECK),
    (15025844, MZONE),
    (4206964, SZONE),
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
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, condition);
    e3.set_cost(duel, cost);
    e3.set_target(duel, target);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_DARK_MAGICIAN) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && a.rp == a.tp && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let spellcaster = |c: CardId, duel: &Duel| c.is_race(duel, RACE_SPELLCASTER);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, spellcaster, (1, 1), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, spellcaster, (1, 1), false, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}
