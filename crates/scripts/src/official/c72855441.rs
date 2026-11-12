// Protector with Eyes of Blue
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 72855441;

// Normal Summoned with a Level 1 LIGHT Tuner (Lunar Rabbit Omajinai) and Blue-Eyes White Dragons in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (47643326, HAND),
    (47643326, HAND),
    (89631139, HAND),
    (89631139, HAND),
];
// The two effects have no description, so each is told apart by the monster it summons.
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(47643326), Check::SpecialSummonedCode(89631139)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, gvtg);
    e2.set_operation(duel, gvop);
    c.register_effect(duel, e2);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_TUNER)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.get_level(duel) == 1
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter1(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter1(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn gvfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_able_to_grave(duel) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && gvfilter(chkc, duel, ft));
    }
    if chk == 0 {
        return Ok(ft > -1
            && duel.is_existing_target(|c, duel| gvfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, |c, duel| gvfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn gvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_grave(tc, REASON_EFFECT)? != 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
