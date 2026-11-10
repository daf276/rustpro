// Barrage Blast
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 51369889;

// Set in the Spell & Trap Zone and in hand, with Superdreadnought Rail Cannon Flying Launcher on the field with Xyz Import
// attaching the opponent's Sangan, and in the Graveyard with the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (ID, GRAVE),
    (38354018, MZONE),
    (38354018, GRAVE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, rmcost);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, damcon);
    e3.set_cost(duel, damcost);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_type(duel, TYPE_XYZ)
}

fn rmcost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 100);
    Ok(true)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    let c = e.get_handler(duel);
    let dg = duel.get_matching_group(rmfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.check_remove_overlay_card_full(tp, 0, 0, 1, REASON_COST, Some(dg))
            && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVEXYZ);
    let rt = duel.get_target_count(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into())) as u16;
    let count = duel.remove_overlay_card_full(tp, 0, 0, 1, rt, REASON_COST, Some(dg))? as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, count, count, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_reason(duel, REASON_BATTLE) || (c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp))
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_race(duel, RACE_MACHINE)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| desfilter(c, duel, tp), 1, None))
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_remove_as_cost(duel) && c.get_rank(duel) > 0
}

fn damcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(damfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, damfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    let rank = g.get_first().map_or(0, |tc| tc.get_rank(duel));
    e.set_label(duel, i64::from(rank));
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let damage = a.e.get_label(duel) as i32 * 200;
    duel.set_target_player(1 - tp);
    duel.set_target_param(damage as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, damage);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}
