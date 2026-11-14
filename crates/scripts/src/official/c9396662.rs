// Mirror Mage of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, MZONE, REMOVED};

const ID: u32 = 9396662;

pub const LISTED_SERIES: &[u16] = &[SET_ICE_BARRIER];

const TOKEN_ICE_BARRIER: u32 = 44308318;

pub const LISTED_NAMES: &[u32] = &[ID, TOKEN_ICE_BARRIER];

// Two on the field (one to release), and Brionac in the banished zone to add to the hand (no Ice Barrier monster of the Main Deck is ported).
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (50321796, REMOVED), (50321796, REMOVED)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, tkcost);
    e1.set_target(duel, tktg);
    e1.set_operation(duel, tkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn tkcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_EFFECT) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn tkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    let filter = |sc, duel: &Duel| tkcostfilter(sc, duel, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_ICE_BARRIER, TYPES_TOKEN, 0, 0, 1, RACE_AQUA, ATTRIBUTE_WATER)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(
            a.e.get_handler(duel).has_level(duel) && duel.is_player_can_special_summon_monster(a.tp, TOKEN_ICE_BARRIER, token_stats())
        );
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !(c.is_type(duel, TYPE_SYNCHRO) && c.is_attribute(duel, ATTRIBUTE_WATER)) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| {
        !c.is_original_type(duel, TYPE_SYNCHRO) || !c.is_original_attribute(duel, ATTRIBUTE_WATER)
    });
    let mut ft = 3.min(duel.get_location_count(tp, LOCATION_MZONE));
    if ft == 0 || !duel.is_player_can_special_summon_monster(tp, TOKEN_ICE_BARRIER, token_stats()) {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let ct = duel.announce_number_range(tp, 1, ft as u32)?;
    for _ in 0..ct {
        if let Some(token) = duel.create_token(tp, TOKEN_ICE_BARRIER) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let ct = duel.special_summon_complete()?;
    if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        c.update_level(duel, ct, None, None);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ICE_BARRIER])
        && c.is_able_to_hand(duel)
        && (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel))
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let location = LOCATION_DECK | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, location as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK | LOCATION_REMOVED)
}
