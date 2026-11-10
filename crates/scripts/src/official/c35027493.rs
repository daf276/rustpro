// Deck Devastation Virus
use super::c54974237::{reset_op, turncon, turnop};
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 35027493;
const PYRO_CLOCK: u32 = 1082946;

// In the hand with Summoned Skull (DARK, 2500 ATK) to Tribute and the opponent's Sangan (1000 ATK) to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (70781052, MZONE), (70781052, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_TOHAND | TIMINGS_CHECK_MONSTER);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_attack_above(duel, 2000)
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

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, 1500)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(tgfilter, a.tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_attack_below(duel, 1500)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let conf = duel.get_field_group(tp, 0, LOCATION_MZONE | LOCATION_HAND);
    if !conf.is_empty() {
        duel.confirm_cards(tp, conf)?;
        let dg = conf.filter(duel, filter, None);
        duel.destroy(dg, REASON_EFFECT)?;
        duel.shuffle_hand(1 - tp)?;
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let reset = RESET_PHASE | PHASE_END | RESET_OPPO_TURN;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DRAW);
    e1.set_operation(duel, desop);
    e1.set_reset_full(duel, reset, 3);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_PHASE | PHASE_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, turncon);
    e2.set_operation(duel, turnop);
    e2.set_reset_full(duel, reset, 3);
    duel.register_effect(e2, tp);
    e2.set_label_object(duel, Some(e1));
    let descnum = if tp == c.get_owner(duel) { 0 } else { 1 };
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_description(duel, aux::stringid(ID, descnum));
    e3.set_property(
        duel,
        EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE,
    );
    e3.set_code(duel, PYRO_CLOCK);
    e3.set_label_object(duel, Some(e2));
    e3.set_owner_player(duel, tp);
    e3.set_operation(duel, reset_op);
    e3.set_reset_full(duel, reset, 3);
    c.register_effect(duel, e3);
    aux::register_client_hint(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, ep) = (a.e, a.ep);
    if ep == e.get_owner_player(duel) {
        return Ok(());
    }
    let hg =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if hg.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(1 - ep, hg)?;
    let dg = hg.filter(duel, filter, None);
    duel.destroy(dg, REASON_EFFECT)?;
    duel.shuffle_hand(ep)?;
    Ok(())
}
