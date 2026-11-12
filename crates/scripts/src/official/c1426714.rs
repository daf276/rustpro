// Number 48: Shadow Lich
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 1426714;
pub const XYZ_NUMBER: u32 = 48;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// In the Extra Deck, Xyz Summoned with two Level 3 monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (32864, MZONE), (1761063, MZONE), (2468169, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID + 1)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_END_PHASE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e2.set_condition(duel, |duel, a| token_count(duel, a.e.get_handler_player(duel)) > 0);
    e2.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(
        duel,
        Value::Card(|duel, _, c| token_count(duel, c.get_controler(duel)) as i32 * 500),
    );
    c.register_effect(duel, e3);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 500, 500, 1, RACE_FIEND, ATTRIBUTE_DARK)
}

fn token_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(|c, duel| c.is_code(duel, ID + 1), tp, LOCATION_ONFIELD, 0, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()) {
        let token = duel.create_token(tp, ID + 1);
        unported!(duel, token.is_none());
        if let Some(token) = token {
            duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
