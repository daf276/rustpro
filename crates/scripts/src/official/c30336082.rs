// Sangen Summoning
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 30336082;

pub const LISTED_SERIES: &[u16] = &[SET_TENPAI_DRAGON];
// Tenpai Dragon Paidra in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (39931513, DECK), (39931513, DECK), (39931513, DECK)];
// Random play spends its Mystical Space Typhoons (in either hand or Set) before the Battle Phase, so this card is never
// destroyed there with a Dragon Synchro Monster on the field (0 in 64 with Ancient Fairy Dragons).
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(a.e.get_handler_player(duel)) && duel.is_phase(PHASE_MAIN1)
    });
    e2.set_target_card(duel, |duel, _, c| {
        c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_DRAGON)
    });
    e2.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner_player(duel) == 1 - e.get_handler_player(duel) && te.is_activated(duel)),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, |duel, _| duel.is_battle_phase());
    e4.set_target(duel, atktg);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TENPAI_DRAGON]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 1, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_SYNCHRO)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g = duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, atk);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        let value = tc.get_attack(duel) * 2;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, value);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
