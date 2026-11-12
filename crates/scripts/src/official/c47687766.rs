// Geargiauger
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 47687766;

pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand to Normal Summon, with Level 4 EARTH Machines in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (2311603, DECK), (7526150, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_level(duel) == 4
        && c.is_race(duel, RACE_MACHINE)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.get_race(duel) != RACE_MACHINE);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}
