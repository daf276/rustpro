// Eradicator Epidemic Virus
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 54974237;
const PYRO_CLOCK: u32 = 1082946;

// In the hand with Summoned Skull (DARK, 2500 ATK) to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (70781052, MZONE), (70781052, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_TOHAND | TIMING_END_PHASE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

pub fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.get_attack(duel) >= 2500
}

pub fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, costfilter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, costfilter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CARDTYPE);
    let decl_type = 2 << duel.select_option(tp, &[DECLTYPE_SPELL, DECLTYPE_TRAP])?;
    a.e.set_label(duel, decl_type);
    let g = duel.get_matching_group(
        |c, duel| c.is_faceup(duel) && c.is_type(duel, decl_type as u32),
        tp,
        0,
        LOCATION_ONFIELD,
        None,
    );
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn cffilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_HAND) || (c.is_facedown(duel) && c.is_spell_trap(duel))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let decl_type = e.get_label(duel);
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD | LOCATION_HAND);
    if !g.is_empty() {
        let cg = g.filter(duel, cffilter, None);
        duel.confirm_cards(tp, cg)?;
        let dg = g.filter(duel, |c, duel| c.is_type(duel, decl_type as u32), None);
        duel.destroy(dg, REASON_EFFECT)?;
        duel.shuffle_hand(1 - tp)?;
    }
    let c = e.get_handler(duel);
    let reset = RESET_PHASE | PHASE_END | RESET_OPPO_TURN;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DRAW);
    e1.set_operation(duel, desop);
    e1.set_label(duel, decl_type);
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
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_reset_full(duel, reset, 3);
    e4.set_target_range(duel, 0, 1);
    duel.register_effect(e4, tp);
    Ok(())
}

pub fn reset_op(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(e2) = a.e.get_label_object(duel) else { return Ok(()) };
    turnop(
        duel,
        &Args {
            e: e2,
            re: Some(a.e),
            ..*a
        },
    )
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
    let decl_type = e.get_label(duel);
    let dg = hg.filter(duel, |c, duel| c.is_type(duel, decl_type as u32), None);
    duel.destroy(dg, REASON_EFFECT)?;
    duel.shuffle_hand(ep)?;
    Ok(())
}

pub fn turncon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
}

pub fn turnop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let ct = e.get_label(duel) + 1;
    e.set_label(duel, ct);
    e.get_handler(duel).set_turn_counter(duel, ct as u16);
    if ct == 3 {
        if let Some(e1) = e.get_label_object(duel) {
            e1.reset(duel);
        }
        if let Some(re) = a.re {
            re.reset(duel);
        }
    }
    Ok(())
}
