// Bird of Paradise Lost
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 84845628;

// On the field with Tuners in the hand to discard and in the Graveyard, and monsters of the opponent's to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (10591919, HAND),
    (10591919, HAND),
    (14558127, GRAVE),
    (549481, MZONE | OPPONENT),
    (1784619, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(adval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_CONTROL);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, cost);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn adval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let tp = c.get_controler(duel);
    duel.get_matching_group_count(|c, duel| c.is_type(duel, TYPE_TUNER), tp, LOCATION_GRAVE, 0, None) as i32 * 100
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let reason_player = duel.get_reason_player();
        duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
    }
    Ok(())
}
