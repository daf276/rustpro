// Rescue-ACE Impulse
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 38339996;

// On the field and in the hand, against the opponent's Ash Blossom and Elfnote Lucina. No "Rescue-ACE" Machine monster is ported,
// so the Special Summon effect is only reported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (14558127, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcond);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_has_effect(duel, EFFECT_CANNOT_TRIGGER).is_none()
}

fn strongest(duel: &Duel, tp: u8) -> Option<CardSet> {
    let g = duel.get_matching_group(cfilter, tp, 0, LOCATION_MZONE, None);
    g.get_max_group(duel, |c, duel| i64::from(c.get_attack(duel))).map(|(tg, _)| tg)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && strongest(duel, a.tp).is_some_and(|tg| !tg.is_empty()))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tg) = strongest(duel, tp).filter(|tg| !tg.is_empty()) else {
        return Ok(());
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let sg = tg.select(duel, tp, 1, 1, None)?;
    if let Some(tc) = sg.first() {
        duel.hint_selection(sg);
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3302);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(
            duel,
            EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_CLIENT_HINT,
        );
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn spcond(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re
            .is_some_and(|re| re.is_monster_effect(duel) && re.get_activate_location(duel) == LOCATION_MZONE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RESCUE_ACE]) && c.is_race(duel, RACE_MACHINE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
