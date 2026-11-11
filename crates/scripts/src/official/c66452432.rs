// Borrelshroud Dragon
use super::c85289965::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 66452432;

// Link Summoned in the Extra Deck with three Rokket Tracers on the field to destroy, and Borrelsword and Breachborrel Dragon
// (Link-4 or lower Borrel monsters) in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (85289965, EXTRA),
    (90011273, EXTRA),
    (68464358, MZONE),
    (68464358, MZONE),
    (68464358, MZONE),
    (68464358, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(!c.is_controler(duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD);
    e1b.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1b.set_code(duel, EFFECT_CANNOT_RELEASE);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_target_range(duel, 0, 1);
    e1b.set_target_card(duel, |duel, e, c| c == e.get_handler(duel));
    e1b.set_value(duel, 1);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e3.set_code(duel, EVENT_PHASE + PHASE_BATTLE_START);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn rokket(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ROKKET])
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && rokket(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rokket, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, rokket, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_chain_limit(|_, _, ep, tp, _| tp == ep, 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_matching_card(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(sc);
    if sc.is_can_be_disabled_by_effect(duel, e, true) {
        sc.negate_effects(duel, e.get_handler(duel), 0, true);
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
        {
            duel.break_effect()?;
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_link_below(duel, 4)
        && c.is_set_card(duel, &[SET_BORREL])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
