// Doombearer Psychopompos
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT, REMOVED};

const ID: u32 = 83116692;

// Copies banished to return at the End Phase, with Monsters and Spells in the opponent's Graveyard and field to compare and send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, REMOVED),
    (ID, REMOVED),
    (3557275, GRAVE | OPPONENT),
    (487395, OPPONENT),
    (487395, MZONE | OPPONENT),
    (12580477, GRAVE | OPPONENT),
    (53129443, OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_REMOVED);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sstg);
    e2.set_operation(duel, ssop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.get_turn_count() == a.e.get_handler(duel).get_turn_id(duel) + 1
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_REMOVED as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn remfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn sstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_possible_operation_info(
        0,
        CATEGORY_REMOVE,
        Some(c.into()),
        2,
        PLAYER_ALL,
        (LOCATION_MZONE | LOCATION_GRAVE) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, Some(c.into()), 2, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn ssop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ctm = duel.get_matching_group_count(CardId::is_monster, tp, 0, LOCATION_GRAVE, None);
    let ctst = duel.get_matching_group_count(CardId::is_spell_trap, tp, 0, LOCATION_GRAVE, None);
    let tg =
        ctm < ctst && c.is_able_to_grave(duel) && duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, None);
    let rem = ctm > ctst
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(aux::necro_valley_filter(remfilter), tp, 0, LOCATION_GRAVE, 1, None);
    if !(tg || rem) {
        return Ok(());
    }
    if rem && duel.remove(c, POS_FACEUP, REASON_EFFECT)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let tc = duel.select_matching_card(1 - tp, aux::necro_valley_filter(remfilter), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    } else if tg && duel.send_to_grave(c, REASON_EFFECT)? > 0 && c.is_location(duel, LOCATION_GRAVE) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let tc = duel.select_matching_card(1 - tp, CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
