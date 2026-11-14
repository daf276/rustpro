// Gladiator Beast Noxious
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 67385964;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand against direct attacks of the opponent's, summoned with Gladiator Beast Gistel or by a tag-out, with Hoplomus in
// the Deck to send and summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (26582143, HAND),
    (4253484, DECK),
    (4253484, DECK),
    (13597785, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, hspcon);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, aux::gbspcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e3.set_cost(duel, Cost::self_to_deck);
    e3.set_target(duel, |duel, a, chk, _| {
        super::c57731460::sptg(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp))
    });
    e3.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| filter(c, duel, a.e, a.tp), 114, true, 0)
    });
    c.register_effect(duel, e3);
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker().is_some_and(|at| at.get_controler(duel) != a.tp) && duel.get_attack_target().is_none()
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 114, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 114, tp, tp, false, false, POS_FACEUP)? != 0
        && let Some(at) = duel.get_attacker()
        && at.can_attack(duel)
        && !at.is_immune_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e1);
        duel.calculate_damage(at, Some(c))?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, ID) && c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 114, tp, false, false)
}
