// Branded Sword
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, REMOVED};

const ID: u32 = 81767888;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ, ID + 1];
pub const LISTED_SERIES: &[u16] = &[SET_BRANDED];
// Copies in the Graveyard to banish and to return Fallen of Albaz from banishment.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, GRAVE),
    (CARD_ALBAZ, REMOVED),
    (CARD_ALBAZ, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID + 1), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 2500, 2000, 8, RACE_DRAGON, ATTRIBUTE_DARK)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BRANDED]) && c.is_spell_trap(duel) && c.is_able_to_remove_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 1);
    let bg = duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(!bg.is_empty() && ft > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()));
    }
    let mut ft = ft.min(bg.len() as i32);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_GRAVE, 0, 1, ft as u8, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_label(duel, g.len() as i64);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let e = a.e;
    if chk == 0 {
        let res = e.get_label(duel) != 0;
        e.set_label(duel, 0);
        return Ok(res);
    }
    let ct = e.get_label(duel) as u8;
    duel.set_operation_info(0, CATEGORY_TOKEN, None, ct, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, ct, a.tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = a.e.get_label(duel);
    if i64::from(duel.get_location_count(tp, LOCATION_MZONE)) < ct || !duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats())
    {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() && ct > 1 {
        return Ok(());
    }
    for _ in 0..ct {
        if let Some(token) = duel.create_token(tp, ID + 1) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_code(duel, CARD_ALBAZ) || (c.is_monster(duel) && c.lists_code(duel, CARD_ALBAZ))) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
    {
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}
