// Orcust Cymbal Skeleton
use super::c57835716::{babelcon, register_dark_lock};
use crate::prelude::*;
use crate::testing::{Check, GRAVE};

const ID: u32 = 21441617;

pub const LISTED_SERIES: &[u16] = &[SET_ORCUST];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Graveyard, with Girsu there to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (69811710, GRAVE), (69811710, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_condition(duel, |duel, a| !babelcon(duel, a));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_condition(duel, babelcon);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ORCUST]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && !c.is_code(duel, ID)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tg = duel.select_target(
        tp,
        |tc, duel| spfilter(tc, duel, e, tp),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, tg, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    register_dark_lock(duel, a, aux::stringid(ID, 1));
    Ok(())
}
