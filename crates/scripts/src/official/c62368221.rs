// Ars Magna of Purification and Corruption
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 62368221;

pub const LISTED_SERIES: &[u16] = &[SET_ARS_MAGNA, SET_POWER_PATRON];
// A second copy banished to be Special Summoned when Purification Power Patron is Link Summoned from the Extra Deck with Dark Magician
// (Normal Monsters), Ars Magna Spells in the Deck to add, and Mystical Space Typhoon Set by the opponent to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, REMOVED),
    (37279096, DECK),
    (90728287, DECK),
    (31822037, MZONE),
    (31822037, EXTRA),
    (31822037, EXTRA),
    (46986414, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_REMOVED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ | TYPE_LINK)), 1, None)
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 2));
    e3a.set_category(duel, CATEGORY_REMOVE);
    e3a.set_type(duel, EFFECT_TYPE_IGNITION);
    e3a.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_count_limit(duel, 1, (ID, 2));
    e3a.set_condition(duel, |duel, a| !citrinitascon(duel, a));
    e3a.set_target(duel, bantg);
    e3a.set_operation(duel, banop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3b.set_code(duel, EVENT_FREE_CHAIN);
    e3b.set_condition(duel, citrinitascon);
    e3b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3b);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARS_MAGNA]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let c = a.e.get_handler(duel);
    aux::register_client_hint_full(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)), 0, 2);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_count_limit(duel, 3, 0);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE == REASON_BATTLE));
    e1.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel)
    });
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    Ok(())
}

fn citrinitascon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, EFFECT_ARS_MAGNA_CITRINITAS).is_some()
}

fn powerpatronfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel) && c.is_faceup(duel)
}

fn spelltrapfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_remove(duel)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel) && chkc.is_able_to_remove(duel));
    }
    let power_patron_count = duel.get_matching_group_count(powerpatronfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(power_patron_count > 0 && duel.is_existing_target(spelltrapfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(
        tp,
        spelltrapfilter,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        power_patron_count as u8,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
