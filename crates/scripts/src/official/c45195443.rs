// Elemental HERO Solid Soldier
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 45195443;

pub const LISTED_SERIES: &[u16] = &[SET_HERO];
pub const LISTED_NAMES: &[u32] = &[ID];
// Summoned from the hand with the Level 4 HERO Destiny HERO - Celestial in the hand to Special Summon, and on the field with Dark Hole
// in the hand to destroy it and Celestial in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (63362460, HAND),
    (63362460, HAND),
    (63362460, GRAVE),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e2);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_HERO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter1(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter1(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.re.is_some_and(|re| re.get_handler(duel).is_spell(duel))
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_HERO])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter2(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
