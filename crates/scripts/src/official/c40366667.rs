// Dominus Impulse
use crate::prelude::*;

const ID: u32 = 40366667;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_has_category(duel, CATEGORY_SPECIAL_SUMMON)) && duel.is_chain_disablable(a.ev as u8)
    });
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop::<ID, { ATTRIBUTE_LIGHT | ATTRIBUTE_EARTH | ATTRIBUTE_WIND }>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, opponent_controls_card);
    c.register_effect(duel, e2);
}

pub fn opponent_controls_card(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.e.get_handler_player(duel), 0, LOCATION_ONFIELD) > 0
}

pub fn set_hand_label_and_infos(duel: &mut Duel, a: &Args) {
    let (e, tp) = (a.e, a.tp);
    let from_hand = e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_handler(duel).is_status(duel, STATUS_ACT_FROM_HAND);
    e.set_label(duel, from_hand.into());
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    let Some(re) = a.re else { return };
    let rc = re.get_handler(duel);
    if rc.is_relate_to_effect(duel, re)
        && rc.is_destructable(duel)
        && duel.is_existing_matching_card(CardId::is_trap, tp, LOCATION_GRAVE, 0, 1, None)
    {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
    }
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    set_hand_label_and_infos(duel, a);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, a.eg, 1, a.tp, 0);
    Ok(false)
}

pub fn disop<const ID: u32, const ATTRIBUTES: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.negate_effect(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
        && duel.is_existing_matching_card(CardId::is_trap, tp, LOCATION_GRAVE, 0, 1, None)
        && let Some(eg) = a.eg
    {
        duel.break_effect()?;
        duel.destroy(eg, REASON_EFFECT)?;
    }
    restrict_attributes::<ID, ATTRIBUTES>(duel, e, tp);
    Ok(())
}

pub fn restrict_attributes<const ID: u32, const ATTRIBUTES: u32>(duel: &mut Duel, e: EffectId, tp: u8) {
    if !(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_label(duel) == 1) {
        return;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && re.get_handler(duel).is_attribute(duel, ATTRIBUTES)),
    );
    duel.register_effect(e1, tp);
}
