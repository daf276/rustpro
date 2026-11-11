// Destiny HERO - Disk Commander
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 56570271;
const MONSTER_REBORN: u32 = 83764718;

// In the hand and the Graveyard, with Monster Reborn to Special Summon it from the Graveyard to draw.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (MONSTER_REBORN, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_previous_location(duel) == LOCATION_GRAVE);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(a.tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 2);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if duel.get_flag_effect(tp, ID) != 0 {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}
