// Swordsoul of Mo Ye
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 20001443;
const SWORDSOUL_GRANDMASTER_CHIXIAO: u32 = 69248256;
const THE_GOLDEN_SWORDSOUL: u32 = 82489470;

pub const LISTED_NAMES: &[u32] = &[TOKEN_SWORDSOUL];

// In the hand and on the field with the Golden Swordsoul as the Tuner, and Swordsoul Grandmaster - Chixiao in the Extra Deck
// to Synchro Summon with it (the Synchro Summon is what sends it to the Graveyard as material).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (THE_GOLDEN_SWORDSOUL, MZONE),
    (SWORDSOUL_GRANDMASTER_CHIXIAO, EXTRA),
    (SWORDSOUL_GRANDMASTER_CHIXIAO, EXTRA),
    (ID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(TOKEN_SWORDSOUL),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, tkcost);
    e1.set_target(duel, tktg);
    e1.set_operation(duel, tkop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, drcon);
    e3.set_target(duel, drtg);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn tkcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_SWORDSOUL]) || (c.is_monster(duel) && c.is_race(duel, RACE_WYRM))) && !c.is_public(duel)
}

fn tkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tkcostfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, tkcostfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

pub fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let stats = MonsterStats::new(SET_SWORDSOUL, TYPES_TOKEN | TYPE_TUNER, 0, 0, 4, RACE_WYRM, ATTRIBUTE_WATER);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, TOKEN_SWORDSOUL, stats));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

pub fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if tktg(duel, a, 0, None)? {
        let c = a.e.get_handler(duel);
        if let Some(token) = duel.create_token(tp, TOKEN_SWORDSOUL) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            e1.set_range(duel, LOCATION_MZONE);
            e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e1.set_absolute_range(duel, tp, 1, 0);
            e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
                c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_SYNCHRO)
            });
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            token.register_effect_full(duel, e1, true);
            let e2 = aux::create_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            token.register_effect_full(duel, e2, true);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

pub fn drcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
