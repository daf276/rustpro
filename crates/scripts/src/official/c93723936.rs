// Scourge of the White Forest
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 93723936;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];
// Diabell and Poplar (White Forest Synchros) to release and Elzette, Azamina (an Illusion) in the Graveyard to Special Summon;
// copies in the hand for Astellar of the White Forest on the field to send as its cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (14307929, MZONE),
    (5800323, MZONE),
    (25592142, MZONE),
    (25592142, MZONE),
    (38648860, GRAVE),
    (38648860, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    // The Lua passes the description to SetCategory, so the effect has none.
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, discost);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_reason(duel, REASON_COST) && a.re.is_some_and(|re| re.is_activated(duel) && re.is_monster_effect(duel))
    });
    e2.set_target(duel, super::c99289828::settg);
    e2.set_operation(duel, super::c99289828::setop);
    c.register_effect(duel, e2);
}

// The Lua's handler argument is nil: CheckReleaseGroupCost shifts it out.
fn discostfilter(c: CardId, duel: &Duel) -> bool {
    let mut eqpg = c.get_equip_group(duel);
    eqpg.insert(c);
    c.is_type(duel, TYPE_SYNCHRO)
        && duel.is_existing_target(|c, duel| c.is_negatable(duel), 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(eqpg))
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, discostfilter, (1, 1), false, None, None));
    }
    let Some(rc) = duel
        .select_release_group_cost(tp, discostfilter, (1, 1), false, None, None)?
        .get_first()
    else {
        return Ok(false);
    };
    let label = i64::from(rc.is_set_card(duel, &[SET_WHITE_FOREST]));
    a.e.set_label(duel, label);
    duel.release(rc, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_negatable(duel) && chkc != c);
    }
    let negatable = |c: CardId, duel: &Duel| c.is_negatable(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    if e.get_label(duel) == 1 {
        duel.set_target_param(1);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    }
    e.set_label(duel, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ILLUSION) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.is_can_be_disabled_by_effect(duel, e, true)) {
        return Ok(());
    }
    tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
    duel.adjust_instantly(Some(tc));
    let filter = aux::necro_valley_filter(move |c, duel| spfilter(c, duel, e, tp));
    let param: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if tc.is_disabled(duel)
        && param > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
