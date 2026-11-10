// Double Headed Anger Knuckle
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 146746;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_MACHINE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Machines (Overdrive, Golgoil), on the field and in the Graveyard, with Level 10 Machines
// (Babeldecker) in the Graveyard to Special Summon and a Machine in the hand to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, GRAVE),
    (2311603, MZONE),
    (7526150, MZONE),
    (2311603, HAND),
    (45116390, GRAVE),
    (45116390, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && c.is_able_to_grave_as_cost(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, tp), tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| costfilter(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[10])
        && c.is_race(duel, RACE_MACHINE)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    // The Lua calls s.filter, which does not exist, so a chkc errors.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e = duel.create_effect(c);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_code(duel, code);
            e.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn costfilter2(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter2(c, duel, tp), tp, LOCATION_ONFIELD | LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| costfilter2(c, duel, tp),
        tp,
        LOCATION_ONFIELD | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
