// Converging Wills Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 291414;

pub const LISTED_SERIES: &[u16] = &[SET_MAJESTIC];
pub const LISTED_NAMES: &[u32] = &[21159309];

// Copies in the Deck to draw and to Special Summon as the Level 1 Dragon, with Ancient Fairy Life Dragon (a Level 8 Synchro Dragon) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, DECK), (ID, DECK), (ID, DECK), (ID, DECK), (43321985, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_set_card(duel, &[SET_MAJESTIC]))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_CODE);
    e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_value(duel, 21159309);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_DRAW);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Ok(chk != 0 || !a.e.get_handler(duel).is_public(duel))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 8) && c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_SYNCHRO)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_level(duel, &[1]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0) {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
