// Infinitrack Tunneller
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 97316367;

// In hand with Infinitrack Harvester on the field to Tribute, and in the Graveyard with five more EARTH Machines.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (35645105, MZONE),
    (35645105, MZONE),
    (ID, GRAVE),
    (ID, GRAVE),
    (35645105, GRAVE),
    (35645105, GRAVE),
    (61912252, GRAVE),
    (61912252, GRAVE),
    (76830505, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_MACHINE)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && (duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_TOFIELD, 0xff) > 0
            || (c.is_controler(duel, tp) && c.get_sequence(duel) < 5))
        && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

pub fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c, duel: &Duel| cfilter(c, duel, tp);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_able_to_deck(duel)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_player_can_draw(tp, 2) && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 5, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 5, 5, None)?;
    let count = g.map_or(0, |g| g.len()) as u8;
    duel.set_operation_info(0, CATEGORY_TODECK, g, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let td = tg
        .unwrap_or_default()
        .filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if tg.is_none() || td.is_empty() {
        return Ok(());
    }
    duel.send_to_deck(td, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let g = duel.get_operated_group();
    if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    let ct = g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if ct > 0 {
        duel.break_effect()?;
        duel.draw(tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}
