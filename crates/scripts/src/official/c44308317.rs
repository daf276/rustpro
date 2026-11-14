// Speaker for the Ice Barriers
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 44308317;

pub const LISTED_NAMES: &[u32] = &[44308318];
pub const LISTED_SERIES: &[u16] = &[SET_ICE_BARRIER];

// Copies in the hand and Graveyard, with Ice Barrier monsters on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (23950192, MZONE),
    (90311614, MZONE),
    (9396662, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_defense_pos(duel));
    e1.set_condition(duel, super::c18482591::con);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, tokentg);
    e3.set_operation(duel, tokenop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_monster(duel) && c.is_set_card(duel, &[SET_ICE_BARRIER]);
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_AQUA, ATTRIBUTE_WATER);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster_full(tp, ID + 1, stats, POS_FACEUP, tp, 0)
}

fn tokentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(can_summon_token(duel, a.tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn tokenop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if can_summon_token(duel, tp)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
