// Primeval Planet Perlereino
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 77103950;

pub const LISTED_SERIES: &[u16] = &[SET_TEARLAMENTS];
pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST];

// In the hand, with Tearlaments monsters in the Deck to search and on the field to hit.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (4928565, DECK), (73956664, DECK), (73956664, MZONE), (92731385, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_TEARLAMENTS]) || c.is_type(duel, TYPE_FUSION)
    });
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_TO_DECK);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && ((c.is_set_card(duel, &[SET_TEARLAMENTS]) && c.is_monster(duel)) || c.is_code(duel, CARD_VISAS_STARFROST))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_GRAVE | LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_set_card(duel, &[SET_TEARLAMENTS])
        && (c.is_monster(duel) || c.is_previous_location(duel, LOCATION_MZONE))
        && c.is_reason(duel, REASON_EFFECT)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
