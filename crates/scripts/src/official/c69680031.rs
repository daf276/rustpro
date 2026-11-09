// Dogmatika Fleurdelis, the Knighted
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 69680031;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with a monster of the opponent's from the Extra Deck, Special Summoned beside the other Dogmatika monster to negate it,
// and attacking with it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (73355772, MZONE),
    (72272462, EXTRA | OPPONENT),
    (24094653, HAND | OPPONENT),
    (62962630, HAND | OPPONENT),
    (15025844, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn dogmatika(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DOGMATIKA])
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_main_phase()
        && duel.is_existing_matching_card(
            |c, duel| c.is_summon_location(duel, LOCATION_EXTRA),
            a.tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false) && duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_negatable_monster, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0
        && duel.is_existing_matching_card(dogmatika, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
        && !g.is_empty()
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(sg);
        let Some(tc) = sg.get_first() else { return Ok(()) };
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker()
        .is_some_and(|at| at.is_controler(duel, a.tp) && at.is_set_card(duel, &[SET_DOGMATIKA]))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(dogmatika, a.tp, LOCATION_MZONE, 0, 1, None))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(dogmatika, a.tp, LOCATION_MZONE, 0, None);
    for tc in g {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
