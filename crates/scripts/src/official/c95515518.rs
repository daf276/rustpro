// The Weather Painter Moonbow
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 95515518;
const RAINBOW: u32 = 54178659;

pub const LISTED_NAMES: &[u32] = &[RAINBOW];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_THE_WEATHER], Some(lc), sumtype, tp)
    }
}

// Link Summoned from three The Weather Painter Rain, with one banished and Rainbow in the Extra Deck, and Raigeki from the opponent
// to destroy it. The granted effect needs an opponent's monster to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (92411493, MZONE),
    (92411493, MZONE),
    (92411493, MZONE),
    (92411493, MZONE),
    (92411493, REMOVED),
    (RAINBOW, EXTRA),
    (12580477, HAND | OPPONENT),
    (6103114, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesGranted(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_MZONE) && c.is_link_summoned(duel)
    });
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, rmcost);
    e4.set_target(duel, rmtg);
    e4.set_operation(duel, rmop);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_range(duel, LOCATION_MZONE, 0);
    e5.set_target_card(duel, |duel, e, c| {
        c.is_type(duel, TYPE_EFFECT) && c.is_set_card(duel, &[SET_THE_WEATHER]) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
    });
    e5.set_label_object(duel, Some(e4));
    c.register_effect(duel, e5);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_THE_WEATHER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter1(c, duel, e, tp), LOCATION_REMOVED)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| spfilter1(c, duel, e, tp), LOCATION_REMOVED, POS_FACEUP)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, RAINBOW)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter2(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Cost::self_banish(duel, a, chk);
    }
    duel.hint(HINT_OPSELECTED, 1 - a.tp, a.e.get_description(duel));
    Cost::self_banish(duel, a, chk)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, 0, REASON_EFFECT | REASON_TEMPORARY)? > 0 {
        let ct = if duel.get_current_phase() <= PHASE_STANDBY { 2 } else { 1 };
        tc.register_flag_effect(
            duel,
            ID,
            RESET_EVENT | (RESETS_STANDARD & !RESET_TEMP_REMOVE) | RESET_PHASE | PHASE_STANDBY,
            0,
            ct,
        );
        let turn = i64::from(duel.get_turn_count());
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, retcon);
        e1.set_operation(duel, retop);
        e1.set_reset_full(duel, RESET_PHASE | PHASE_STANDBY, ct);
        e1.set_label(duel, turn);
        e1.set_label_object_group(duel, Some(tc.into()));
        duel.register_effect(e1, tp);
        tc.set_reason_effect(duel, e1);
    }
    Ok(())
}

fn retcon(duel: &Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    i64::from(duel.get_turn_count()) > a.e.get_label(duel) && tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) > 0)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_SELECTMSG, 1 - a.tp, HINTMSG_ZONE);
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(tc)?;
    }
    Ok(())
}
