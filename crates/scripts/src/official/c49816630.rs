// Gaiarmor Dragonshell
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 49816630;

// Copies in the Extra Deck, Synchro Summoned from Guardragon Justicia (Level 2 Tuner) and Prevent Rat (Level 4), and in the
// Graveyard to return and equip, with attackers on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, GRAVE),
    (11012154, MZONE),
    (11012154, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOEXTRA);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, drcon);
    e3.set_target(duel, stock::draw_target::<1>);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_SYNCHRO) && c.is_able_to_extra(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 3, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 3, 3, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOEXTRA, g, 3, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_summon_location(duel, LOCATION_EXTRA)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(eqfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    let c = a.e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && duel.equip(tp, c, tc)? {
        let e1 = duel.create_effect(tc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(|duel, e, c| i32::from(e.get_owner(duel) == c)));
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .is_some_and(|ec| ec.is_relate_to_battle(duel))
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
