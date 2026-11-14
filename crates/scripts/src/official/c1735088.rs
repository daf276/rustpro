// Wind-Up Zenmaintenance
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 1735088;

pub const LISTED_SERIES: &[u16] = &[SET_WIND_UP];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_WIND_UP], Some(lc), sumtype, tp)
    }
}

// Link Summoned from Wind-Up Soldier, with more in the Deck to search and summon, Wind-Up Zenmaines to attach to, and Raigeki of the
// opponent to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (12299841, MZONE),
    (12299841, MZONE),
    (12299841, DECK),
    (12299841, DECK),
    (12299841, DECK),
    (12299841, DECK),
    (78156759, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_DESTROY));
    e3.set_target(duel, mattg);
    e3.set_operation(duel, matop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WIND_UP]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, 100);
    Ok(chk == 0)
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_WIND_UP])
        && c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_code(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    e.set_label(duel, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEDOWN, REASON_COST)?;
    e.set_label_object_group(duel, Some(g));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let code = tc.get_code(duel);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_WIND_UP]) && c.is_type(duel, TYPE_XYZ)
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && matfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(matfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, matfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        duel.overlay(tc, c)?;
    }
    Ok(())
}
