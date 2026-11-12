// Number C5: Chaos Chimera Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 69757518;
pub const XYZ_NUMBER: u32 = 5;

// In the Extra Deck, Xyz Summoned with three Level 6 Normal Monsters, with cards in both Graveyards and the opponent's monsters to
// attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (8353769, MZONE),
    (9540040, MZONE),
    (11761845, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (13069066, GRAVE),
    (14015067, GRAVE | OPPONENT),
];
// The chain attack needs the Xyz Summoned copy to attack and keep a material, which only 1 of 64 scenarios reaches.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 1000));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_condition(duel, atcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, cost);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) && duel.check_mut(|duel| c.can_chain_attack_full(duel, 0, true))
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_battle(duel) {
        return Ok(());
    }
    duel.chain_attack();
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE | PHASE_DAMAGE_CAL);
    c.register_effect(duel, e1);
    Ok(())
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let half = (duel.get_lp(a.tp) / 2) as u32;
    duel.pay_lp_cost(a.tp, half)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ)
            && duel.is_existing_target(CardId::is_able_to_deck, tp, LOCATION_GRAVE, LOCATION_GRAVE, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, CardId::is_able_to_deck, tp, LOCATION_GRAVE, LOCATION_GRAVE, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 2, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let sg = g.unwrap_or_default().filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if sg.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let sg1 = sg.select(duel, tp, 1, 1, None)?;
        let c = e.get_handler(duel);
        if duel.send_to_deck(sg1, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? != 0 && c.is_relate_to_effect(duel, e) {
            duel.overlay(c, sg.difference(sg1))?;
        }
    }
    Ok(())
}
