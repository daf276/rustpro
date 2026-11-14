// White Rose Cloister
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 84335863;

pub const LISTED_SERIES: &[u16] = &[SET_ROSE_DRAGON];
// Copies to activate, a Plant (Mushroom Man) and Red Rose Dragon in the hand to summon
// once the opponent's Raigeki or Dark Hole clears the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (14181608, HAND),
    (26118970, HAND),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PREDRAW);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(a.tp) && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 0
    });
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_ROSE_DRAGON]) || c.is_race(duel, RACE_PLANT)) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_CARDTYPE);
    let option = duel.select_option(a.tp, &[DECLTYPE_MONSTER, DECLTYPE_SPELL, DECLTYPE_TRAP])?;
    a.e.set_label(duel, i64::from(option));
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(top_card) = duel.get_decktop_group(tp, 1).get_first() else {
        return Ok(());
    };
    duel.confirm_decktop(tp, 1)?;
    let declared_type = match a.e.get_label(duel) {
        0 => TYPE_MONSTER,
        1 => TYPE_SPELL,
        _ => TYPE_TRAP,
    };
    if top_card.is_type(duel, declared_type) {
        let c = a.e.get_handler(duel);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_level_above(duel, 7) && c.is_synchro_monster(duel));
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}
