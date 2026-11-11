// Conduction Warrior Linear Magnum Plus Minus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 44839512;

// Alpha, Beta and Gamma the Magnet Warrior (Normal Monsters) to send, and Beta on the field for the ATK gain.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (99785935, HAND), (39256679, DECK), (11549357, DECK), (39256679, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 1));
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 2));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_DESTROY));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGNET_WARRIOR])
        && c.is_monster(duel)
        && c.get_original_level(duel) <= 4
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
}

fn cost_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(spconfilter, tp, LOCATION_HAND | LOCATION_MZONE | LOCATION_DECK, 0, None)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    aux::select_unselect_group_check(duel, cost_group(duel, tp), 2, 2, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = cost_group(duel, tp);
    let g = aux::select_unselect_group(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_TOGRAVE)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.has_non_zero_attack(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && atkfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    let attack = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel) / 2);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(c.into()), 1, tp, attack);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
    {
        let attack = tc.get_attack(duel) / 2;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, attack);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
