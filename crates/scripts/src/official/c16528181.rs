// King's Sarcophagus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 16528181;
const IMSETY: u32 = 84941194;

// Active on the field and in the hand, with Imsety (the only ported Horus monster) in the Deck and on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, HAND), (IMSETY, DECK), (IMSETY, DECK), (IMSETY, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_HORUS]));
    e2.set_value(duel, Value::EffectPlayerCard(indvalue));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 4, ID);
    e3.set_cost(duel, dtgcost);
    e3.set_target(duel, dtgtg);
    e3.set_operation(duel, dtgop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BATTLE_START);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, btgcond);
    e4.set_target(duel, btgtg);
    e4.set_operation(duel, btgop);
    c.register_effect(duel, e4);
}

fn indvalue(duel: &Duel, _: EffectId, re: EffectId, _: u8, c: CardId) -> bool {
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return true;
    }
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    !g.is_some_and(|g| g.is_contains(c))
}

fn dtgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_able_to_grave_as_cost, 1, 1, REASON_COST)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HORUS]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn dtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn dtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn btgcond(duel: &Duel, a: &Args) -> bool {
    let (attacker, target) = duel.get_battle_monster(a.tp);
    target.is_some() && attacker.is_some_and(|a| a.is_set_card(duel, &[SET_HORUS]) && a.is_faceup(duel))
}

fn btgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (_, target) = duel.get_battle_monster(a.tp);
    if chk == 0 {
        return Ok(target.is_some_and(|at| at.is_able_to_grave(duel)));
    }
    if let Some(at) = target {
        a.e.set_label_object_group(duel, Some(at.into()));
        duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(at.into()), 1, a.tp, 0);
    }
    Ok(false)
}

fn btgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.is_relate_to_battle(duel)
        && tc.is_controler(duel, 1 - tp)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
