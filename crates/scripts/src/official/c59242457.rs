// Exosisters Magnifica
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 59242457;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, xyz, sumtype, tp| {
        c.is_type_full(duel, TYPE_XYZ, Some(xyz), sumtype, tp)
            && c.is_rank(duel, &[4])
            && c.is_set_card_full(duel, &[SET_EXOSISTER], Some(xyz), sumtype, tp)
    });
    const LV: Option<u32> = None;
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Rank 4 Exosisters (Karmael), with the opponent's Dark Hole and Raigeki to chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (77675029, MZONE),
    (77675029, MZONE),
    (77675029, MZONE),
    (77675029, EXTRA),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(
        duel,
        Value::SpSummonCondition(|_, _, se, _, st, _, _| st & SUMMON_TYPE_XYZ == SUMMON_TYPE_XYZ && se.is_none()),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e3.set_target(duel, tetg);
    e3.set_operation(duel, teop);
    c.register_effect(duel, e3);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let description = e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - tp, description);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn tefilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_owner(duel) == tp && c.is_type(duel, TYPE_XYZ) && c.is_able_to_extra(duel)
}

fn tetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0
            && c.get_overlay_group(duel)
                .is_exists(duel, |mc, duel| tefilter(mc, duel, tp), 1, None));
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    let description = e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - tp, description);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_OVERLAY as i32);
    Ok(false)
}

fn teop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = c
        .get_overlay_group(duel)
        .filter_select(duel, tp, |mc, duel| tefilter(mc, duel, tp), 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? < 1 || !tc.is_location(duel, LOCATION_EXTRA) {
        return Ok(());
    }
    if c.is_facedown(duel) || c.is_controler(duel, 1 - tp) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if pg.len() > 1 || (pg.len() == 1 && !pg.contains(c)) {
        return Ok(());
    }
    if tc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && c.is_can_be_xyz_material(duel, Some(tc), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), tc) > 0
        && tc.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        tc.set_material(duel, Some(c.into()));
        duel.overlay(tc, c)?;
        if duel.special_summon(tc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
            tc.complete_procedure(duel);
        }
    }
    Ok(())
}
