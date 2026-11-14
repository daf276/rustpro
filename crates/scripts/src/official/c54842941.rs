// Enneacraft - Atori.MAR
use super::c28454232::{flip_quick, register_counter, register_destroy, register_set_summon};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 54842941;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone and face-down on the field, with the opponent's Mage Power to negate and Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (83746708, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_destroy::<ID>(duel, c);
    register_set_summon::<ID>(duel, c);
    let property = EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL;
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_NEGATE + CATEGORY_DESTROY, property);
    e4.set_condition(duel, negcon);
    e4.set_target(duel, negtg);
    e4.set_operation(duel, negop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_FLIP);
    e5.set_target(duel, postg);
    e5.set_operation(duel, posop);
    c.register_effect(duel, e5);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    duel.is_turn_player(a.tp)
        && a.e.get_handler(duel).is_facedown(duel)
        && a.rp == 1 - a.tp
        && loc & LOCATION_ONFIELD > 0
        && duel.is_chain_negatable(ev)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, a.tp, 0);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let (Some(re), Some(eg)) = (a.re, a.eg)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(eg, REASON_EFFECT)?;
    }
    Ok(())
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_POSITION, None, 1, 1 - a.tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_can_turn_set, a.tp, 0, LOCATION_MZONE, None);
    if g.is_empty() || duel.change_position(g, POS_FACEDOWN_DEFENSE)? == 0 {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g.filter(duel, |c, duel| c.is_position(duel, POS_FACEDOWN_DEFENSE), None) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3313);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
