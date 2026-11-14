// Rite of Aramesir
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 3285551;

pub const LISTED_NAMES: &[u32] = &[TOKEN_ADVENTURER, 39568067];
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(TOKEN_ADVENTURER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| !adventurer_exists(duel, a.tp, TOKEN_ADVENTURER));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|duel, re, _, _| actfilter(duel, re)));
}

fn actfilter(duel: &Duel, re: EffectId) -> bool {
    let rc = re.get_handler(duel);
    !(re.is_monster_effect(duel) && rc.is_on_field(duel) && !rc.is_special_summoned(duel))
}

fn adventurer_exists(duel: &Duel, tp: u8, code: u32) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, code)),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, a.tp, ACTIVITY_CHAIN) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| !actfilter(duel, re)));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_monster(
                tp,
                TOKEN_ADVENTURER,
                MonsterStats::new(0, TYPES_TOKEN, 2000, 2000, 4, RACE_FAIRY, ATTRIBUTE_EARTH),
            ));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, 39568067) && !c.is_forbidden(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !sptg(duel, a, 0, None)? {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, TOKEN_ADVENTURER) else {
        return Ok(());
    };
    if duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && !adventurer_exists(duel, tp, 39568067)
        && duel.is_existing_matching_card(plfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            duel.break_effect()?;
            duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
        }
    }
    Ok(())
}
