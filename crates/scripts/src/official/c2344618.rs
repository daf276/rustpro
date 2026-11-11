// Lunalight Masquerade
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 2344618;
const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const LUNALIGHT_SILVER_HOUND: u32 = 35763582;
const LUNALIGHT_PERFUME_DANCER: u32 = 81196066;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

// Activated from the hand to send from the Deck; Polymerization with two Lunalights Fusion Summons Perfume Dancer, then
// returns to be used again with the materials in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_POLYMERIZATION, HAND),
    (LUNALIGHT_SILVER_HOUND, HAND),
    (LUNALIGHT_BLACK_SHEEP, HAND),
    (LUNALIGHT_SILVER_HOUND, DECK),
    (LUNALIGHT_BLACK_SHEEP, DECK),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_cost(duel, actreg);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn actreg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn thconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_fusion_summoned(duel) && c.is_summon_player(duel, tp) && c.is_faceup(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| thconfilter(c, duel, a.tp), 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    if duel.is_existing_matching_card(
        |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
        tp,
        LOCATION_HAND,
        0,
        1,
        None,
    ) && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        if duel.discard_hand(tp, |_, _| true, 1, 1, REASON_EFFECT | REASON_DISCARD)? == 0 {
            return Ok(());
        }
        let c = a.e.get_handler(duel);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_EXTRA_FUSION_MATERIAL);
        e1.set_count_limit(duel, 1, 0);
        e1.set_target_range(duel, LOCATION_GRAVE, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_able_to_remove(duel) && c.is_monster(duel));
        e1.set_fusion_material_operation(duel, Fusion::banish_material);
        e1.set_value(
            duel,
            Value::Card(|duel, e, c| i32::from(c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_controler(duel, e.get_handler_player(duel)))),
        );
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}
