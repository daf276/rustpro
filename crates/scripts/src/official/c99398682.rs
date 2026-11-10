// Crimson Call
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 99398682;

pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];
// In the hand to search or revive a Level 4 Fiend (Zure, Knight of Dark World) and a copy in the Graveyard to banish when
// Red Dragon Archfiend attacks Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (7459013, GRAVE),
    (7459013, DECK),
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
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, atkcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_FIEND) && c.is_able_to_hand(duel)
}

fn rdafilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND) || (c.is_synchro_monster(duel) && c.lists_code(duel, CARD_RED_DRAGON_ARCHFIEND)))
        && c.is_faceup(duel)
}

fn locations(duel: &Duel, tp: u8) -> u32 {
    if duel.is_existing_matching_card(rdafilter, tp, LOCATION_ONFIELD, 0, 1, None) {
        LOCATION_GRAVE | LOCATION_DECK
    } else {
        LOCATION_GRAVE
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = locations(duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, locations as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let locations = locations(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(thfilter), tp, locations, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let Some(ac) = duel.get_attacker() else { return false };
    ac.is_controler(duel, a.tp)
        && duel.check_mut(|duel| ac.can_chain_attack(duel))
        && (ac.is_code(duel, CARD_RED_DRAGON_ARCHFIEND) || ac.is_set_card(duel, &[SET_ASSAULT_MODE]))
}
