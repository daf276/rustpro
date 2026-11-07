// Destiny HERO - Celestial
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 63362460;

// On the field to attack, with face-up Spells to destroy on the opponent's field (Fusion Gate, Elfnotes: Welcome Home); in the
// Graveyard with a second copy to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (ID, GRAVE),
    (33550694, OPPONENT),
    (64491754, SZONE | OPPONENT),
    (64491754, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, drcon);
    e2.set_cost(duel, drcost);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && desfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.destroy(tc, REASON_EFFECT)? != 0
    {
        duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
    }
    Ok(())
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_HAND, 0) == 0 && aux::exccon(duel, a)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(Cost::self_banish(duel, a, 0)? && duel.is_existing_matching_card(cfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, cfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2));
    }
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
