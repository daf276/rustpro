// Tantrum Toddler
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 7020743;

// In the Extra Deck, Xyz Summoned with Level 1 Normal Monsters (Key Mace, White Duston), with Raigeki in hand for the opponent's set
// Solemn Judgment to negate, and monsters in the Graveyard to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (3557275, MZONE),
    (3557275, GRAVE),
    (3557275, GRAVE),
    (12580477, HAND),
    (12580477, HAND),
    (41420027, SZONE | OPPONENT),
    (41420027, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<1, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CHAIN_NEGATED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e1.set_target(duel, atchtg);
    e1.set_operation(duel, atchop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 700));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) >= 4);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_DESTROY);
    e6.set_type(duel, EFFECT_TYPE_QUICK_O);
    e6.set_code(duel, EVENT_FREE_CHAIN);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e6.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) >= 8);
    e6.set_cost(duel, Cost::detach_from_self::<4, 4>);
    e6.set_target(duel, destg);
    e6.set_operation(duel, desop);
    c.register_effect(duel, e6);
}

fn atchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ) && duel.get_field_group_count(0, LOCATION_GRAVE, LOCATION_GRAVE) > 0);
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, PLAYER_ALL, LOCATION_GRAVE as i32);
    Ok(false)
}

fn atchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_field_group(0, LOCATION_GRAVE, LOCATION_GRAVE);
    let sg = aux::select_unselect_group(
        duel,
        g,
        1,
        2,
        &aux::dpcheck(|c, duel| i64::from(c.get_controler(duel))),
        tp,
        HINTMSG_ATTACH,
    )?;
    if !sg.is_empty() {
        duel.hint_selection_full(sg, true);
        duel.overlay(c, sg)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(|_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(|_, _| true, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
