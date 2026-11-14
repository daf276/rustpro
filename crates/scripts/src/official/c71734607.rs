// Rikka Petal
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 71734607;

pub const LISTED_SERIES: &[u16] = &[SET_RIKKA];
pub const LISTED_NAMES: &[u32] = &[ID];
// On the field with Rikka monsters (Mudan, Snowdrop, Primula) in the Deck, and in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (ID, GRAVE),
    (71002019, DECK),
    (33491462, DECK),
    (8129306, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RIKKA]) && c.is_monster(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel)) && !c.is_code(duel, ID)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    aux::to_hand_or_else(
        duel,
        tc,
        tp,
        CardId::is_able_to_grave,
        |duel| duel.send_to_grave(tc, REASON_EFFECT).map(drop),
        574,
    )?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_PLANT));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    duel.is_turn_player(1 - a.tp)
        && (g.is_empty() || g.filter_count(duel, aux::faceup_filter(|c, duel| c.is_race(duel, RACE_PLANT)), None) == g.len())
}
