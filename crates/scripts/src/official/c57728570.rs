// Crush Card Virus
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 57728570;

// In the hand with Sangan (DARK, 1000 ATK) to Tribute, against Lucina on the field and in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_TOHAND);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_attack_below(duel, 1000)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, costfilter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, costfilter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_above(duel, 1500)
}

fn hgfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_public(duel) || filter(c, duel)
}

fn fgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || filter(c, duel)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_HAND) && c.is_public(duel) || c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel)) && filter(c, duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(hgfilter, tp, 0, LOCATION_HAND, 1, None)
            || duel.is_existing_matching_card(fgfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(tgfilter, tp, 0, LOCATION_MZONE | LOCATION_HAND, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let conf = duel.get_field_group(tp, 0, LOCATION_MZONE | LOCATION_HAND);
    let mut ct = 0;
    if !conf.is_empty() {
        duel.confirm_cards(tp, conf)?;
        let dg = conf.filter(duel, filter, None);
        ct = duel.destroy(dg, REASON_EFFECT)?;
        duel.shuffle_hand(1 - tp)?;
    }
    let g = duel.get_matching_group(filter, 1 - tp, LOCATION_DECK, 0, None);
    if ct > 0 && !g.is_empty() && duel.select_yes_no(1 - tp, aux::stringid(ID, 0))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_DESTROY);
        let dg = g.select(duel, 1 - tp, 1, 3, None)?;
        duel.destroy(dg, REASON_EFFECT)?;
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_target_range(duel, 0, 1);
        e1.set_value(duel, 0);
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e1, tp);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
        e2.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e2, tp);
        let e3 = duel.create_effect(c);
        e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e3.set_description(duel, aux::stringid(ID, 1));
        e3.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        e3.set_target_range(duel, 0, 1);
        duel.register_effect(e3, tp);
    }
    Ok(())
}
