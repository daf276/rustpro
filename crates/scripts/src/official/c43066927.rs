// Mimighoul Fairy
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 43066927;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// Copies face-down on the field to Flip with the opponent's Elfnote Lucina to attack them, and in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_F);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, |duel, a, chk, _| {
        super::c11677278::selfsptg_with(duel, a, chk, controls_mimighoul)
    });
    e2.set_operation(duel, |duel, a| super::c11677278::selfspop_with(duel, a, ID, controls_mimighoul));
    c.register_effect(duel, e2);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    if c.is_able_to_change_controler(duel) {
        duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, a.tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    super::c81522098::give_control(duel, e, tp, true)
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, tp: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel)
        && rc.is_on_field(duel)
        && rc.is_special_summoned(duel)
        && rc.is_summon_location(duel, LOCATION_HAND)
        && rc.is_previous_controler(duel, tp)
}

fn controls_mimighoul(duel: &Duel, tp: u8) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_MIMIGHOUL]));
    duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
}
