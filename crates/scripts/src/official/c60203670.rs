// Gold Pride - Eliminator
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 60203670;

pub const LISTED_SERIES: &[u16] = &[SET_GOLD_PRIDE];

const GOLD_PRIDE_TOKEN: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[GOLD_PRIDE_TOKEN];

// In the hand and on the field with Gold Pride - Leon in the hand, on the field and in the Graveyard to banish for the Token's Level.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (23512906, HAND), (23512906, MZONE), (23512906, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(GOLD_PRIDE_TOKEN),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_lp(a.tp) < duel.get_lp(1 - a.tp));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, tkncost);
    e2.set_target(duel, tkntg);
    e2.set_operation(duel, tknop);
    c.register_effect(duel, e2);
}

fn token_stats(level: u32) -> MonsterStats {
    MonsterStats::new(SET_GOLD_PRIDE, TYPES_TOKEN, 0, 0, level, RACE_MACHINE, ATTRIBUTE_DARK)
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOLD_PRIDE])
        && c.has_level(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_ONFIELD))
        && c.is_able_to_remove_as_cost(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.check_mut(|duel| duel.is_player_can_special_summon_monster(tp, GOLD_PRIDE_TOKEN, token_stats(c.get_level(duel))))
}

fn tkncost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let locations = LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, tp), tp, locations, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| costfilter(c, duel, tp), tp, locations, 0, 1, 1, Some(c.into()))?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    let level = g.get_first().map_or(0, |tc| tc.get_level(duel));
    e.set_label(duel, i64::from(level));
    Ok(false)
}

fn tkntg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn tknop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
        e1.set_value(duel, Value::Material(matlimit));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    let lv = e.get_label(duel) as u32;
    if !(duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, GOLD_PRIDE_TOKEN, token_stats(lv)))
    {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, GOLD_PRIDE_TOKEN) else {
        return Ok(());
    };
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CHANGE_LEVEL_FINAL);
    e2.set_value(duel, lv as i32);
    e2.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    token.register_effect(duel, e2);
    duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn matlimit(duel: &Duel, _: EffectId, c: Option<CardId>, sumtype: u64, _: u8) -> bool {
    let Some(c) = c else { return false };
    [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK]
        .map(u64::from)
        .contains(&sumtype)
        && !c.is_set_card(duel, &[SET_GOLD_PRIDE])
}
