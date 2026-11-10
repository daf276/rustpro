// Lord of the Heavenly Prison
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, SZONE};

const ID: u32 = 9822220;

// Two in hand, and Dark Hole set on the field on both sides (activated from the Set position, on the opponent's turn while it is
// revealed) and in the Deck (a Spell to Set). Both effects share their once per turn, so the reveal comes a turn earlier.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (53129443, SZONE),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
    (53129443, DECK),
    (53129443, DECK),
    (5318639, DECK),
    (5318639, DECK),
    (5318639, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| Ok(chk == 0 && !a.e.get_handler(duel).is_public(duel)));
    e1.set_operation(duel, revop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_label(duel, 0);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn isrev(c: CardId, duel: &Duel) -> bool {
    c.is_public(duel) && c.get_flag_effect(duel, ID) > 0
}

fn revop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 2);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PUBLIC);
    e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
    e2.set_condition(duel, |duel, a| isrev(a.e.get_handler(duel), duel));
    e2.set_target_card(duel, |duel, _, c| c.is_facedown(duel));
    e2.set_value(duel, 1);
    e2.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
    c.register_effect(duel, e2);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    !re.get_handler(duel).is_status(duel, STATUS_ACT_FROM_HAND)
        && re.is_active_type(duel, TYPE_SPELL | TYPE_TRAP)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let label = c.is_public(duel) as i64;
    e.set_label(duel, label);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_SPELL | TYPE_TRAP) && c.is_ssetable(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let wasrev = e.get_label(duel) == 1;
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && wasrev
        && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let Some(tc) = duel
            .select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.break_effect()?;
        if duel.sset(tp, tc)? == 0 {
            return Ok(());
        }
        let turn = duel.get_turn_count();
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_label(duel, turn as i64 + 1);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition_mut(duel, rmcon);
        e1.set_operation(duel, rmop);
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e1, tp);
        tc.create_effect_relation(duel, e1);
    }
    Ok(())
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_some_and(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.get_turn_count() as i64 == a.e.get_label(duel)
    } else {
        a.e.reset(duel);
        false
    }
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
