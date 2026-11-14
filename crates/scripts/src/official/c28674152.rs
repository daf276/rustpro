// Radian, the Multidimensional Kaiju
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 28674152;

pub const LISTED_SERIES: &[u16] = &[SET_KAIJU];
pub const COUNTER_LIST: &[u16] = &[COUNTER_KAIJU];

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// In the hand with a copy on the opponent's field to Tribute, Kyoutou Waterfront for Kaiju Counters, and Raigeki and Dark Hole to fill it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE | OPPONENT),
    (56111151, HAND),
    (12580477, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::add_kaiju_procedure(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, aux::kaiju_counter_cost::<2>);
    e3.set_target(duel, tktg);
    e3.set_operation(duel, tkop);
    c.register_effect(duel, e3);
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 2800, 0, 7, RACE_FIEND, ATTRIBUTE_DARK);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, stats)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if can_summon_token(duel, tp)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, 1);
        token.register_effect_full(duel, e1, true);
    }
    Ok(())
}
