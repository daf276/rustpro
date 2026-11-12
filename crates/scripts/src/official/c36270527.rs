// Ars Magna of Infinity and Finity
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 36270527;

// A second copy banished to be Special Summoned when Purification Power Patron is Link Summoned from the Extra Deck with Dark Magician
// (Normal Monsters), Ars Magna monsters in the Deck to add, Ars Magna - "Citrinitas" in hand for the Quick Effect, and Sangan of
// the opponent to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, REMOVED),
    (62368221, DECK),
    (99753860, DECK),
    (31822037, MZONE),
    (31822037, EXTRA),
    (31822037, EXTRA),
    (46986414, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (37279096, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, thatktg);
    e1.set_operation(duel, thatkop);
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
        a.eg.unwrap_or_default().is_exists(
            duel,
            aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_FUSION | TYPE_LINK)),
            1,
            None,
        )
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
    e3a.set_condition(duel, |duel, a| bancon(duel, a) && !citrinitascon(duel, a));
    e3a.set_target(duel, bantg);
    e3a.set_operation(duel, banop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3b.set_code(duel, EVENT_FREE_CHAIN);
    e3b.set_condition(duel, |duel, a| bancon(duel, a) && citrinitascon(duel, a));
    e3b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3b);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARS_MAGNA]) && c.is_monster(duel) && c.is_race_except(duel, RACE_WARRIOR) && c.is_able_to_hand(duel)
}

fn thatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, None, 1, tp, 0);
    Ok(false)
}

fn thatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel)
    });
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_base_attack(duel) * 3));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn citrinitascon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, EFFECT_ARS_MAGNA_CITRINITAS).is_some()
}

fn powerpatronfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel) && c.is_faceup(duel)
}

fn bancon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(powerpatronfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
