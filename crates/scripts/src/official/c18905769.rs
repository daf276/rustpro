// Guivre
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 18905769;
const TOKEN_WYVERN: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_WYVERN];

// On the field with Dragons in the hand to Tribute Summon and the opponent's Sanganss to destroy by battle for the Token.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (11091375, HAND),
    (11091375, HAND),
    (11091375, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(TOKEN_WYVERN),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| {
            let dragon = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_DRAGON));
            400 * duel.get_matching_group_count(dragon, e.get_handler_player(duel), LOCATION_MZONE, 0, None) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_target(duel, tokentg);
    e2.set_operation(duel, tokenop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, _| duel.is_battle_phase() || duel.is_main_phase2());
    e3.set_target(duel, tribsumtg);
    e3.set_operation(duel, tribsumop);
    e3.set_hint_timing(duel, 0, TIMING_BATTLE_END | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e3);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 400, 400, 1, RACE_DRAGON, ATTRIBUTE_LIGHT)
}

fn tokentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, TOKEN_WYVERN, token_stats())
        );
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tokenop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, TOKEN_WYVERN, token_stats())
        && let Some(token) = duel.create_token(tp, TOKEN_WYVERN)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tribsumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_summonable_full(duel, true, None, 1, 0x1f)
}

fn tribsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tribsumfilter, LOCATION_HAND, CATEGORY_SUMMON)
}

fn tribsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, tribsumfilter, tp, LOCATION_HAND, 0, 1, 1, None)?
        .get_first()
    {
        duel.summon_full(tp, sc, true, None, 1, 0x1f)?;
    }
    Ok(())
}
