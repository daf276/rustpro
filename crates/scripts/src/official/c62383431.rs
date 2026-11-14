// Keurse, the Ogdoadic Light
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 62383431;

pub const LISTED_SERIES: &[u16] = &[SET_OGDOADIC];
// In the GY with monsters to Tribute, Nunu in the GY to summon on its summon, Sangan in the opponent's GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, HAND),
    (26202165, MZONE),
    (26202165, MZONE),
    (36010310, GRAVE),
    (36010310, GRAVE),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost::<1>);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

// Also Aron's cost, which Tributes COUNT.
pub fn spcost<const COUNT: usize>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let check = aux::release_check_mmz;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (COUNT, COUNT), false, Some(&check), None));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (COUNT, COUNT), false, Some(&check), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, 1 - tp), tp, 0, LOCATION_GRAVE, 1, None)
        && duel.select_yes_no(1 - tp, aux::stringid(ID, 2))?)
    {
        return Ok(());
    }
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, 1 - tp));
    let sg = duel.select_matching_card(1 - tp, filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    if let Some(tc) = sg.get_first() {
        duel.special_summon_step(tc, 0, 1 - tp, 1 - tp, false, false, POS_FACEUP)?;
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_OGDOADIC]) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter2(chkc, duel, e, tp));
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

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let c = e.get_handler(duel);
        super::c36010310::banish_on_leave(duel, c, tc);
    }
    Ok(())
}
