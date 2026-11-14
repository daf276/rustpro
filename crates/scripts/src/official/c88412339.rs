// Abyss Actor - Extras
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT, PZONE};

const ID: u32 = 88412339;

pub const LISTED_SERIES: &[u16] = &[SET_ABYSS_ACTOR];
// In the Pendulum Zone and on the field with copies in the Deck to place, and monsters of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (6103114, MZONE | OPPONENT),
    (6103114, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, pentg);
    e2.set_operation(duel, penop);
    c.register_effect(duel, e2);
}

fn penfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ABYSS_ACTOR]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(penfilter, a.tp, LOCATION_DECK, 0, 1, None) && duel.check_pendulum_zones(a.tp))
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, penfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_ABYSS_ACTOR]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 0);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    e3.set_target_range(duel, 1, 0);
    duel.register_effect(e3, tp);
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    rc.is_code(duel, ID)
        && re.get_active_type(duel) == TYPE_PENDULUM + TYPE_SPELL
        && !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && rc.is_location(duel, LOCATION_PZONE)
}
