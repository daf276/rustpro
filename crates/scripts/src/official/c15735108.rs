// Spell Card "Soul Exchange"
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 15735108;

// In the hand with Level 5 Normal Monsters (Shapesnatch) in both hands to Tribute Summon, and Sangan on the field of each
// player to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (4035199, HAND),
    (4035199, HAND),
    (4035199, HAND | OPPONENT),
    (4035199, HAND | OPPONENT),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_operation(duel, opsumop);
    c.register_effect(duel, e2);
}

fn sumfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    duel.check_mut(|duel| {
        let e1 = duel.create_effect(ec);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(
            duel,
            EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SET_AVAILABLE,
        );
        e1.set_code(duel, EFFECT_ADD_EXTRA_TRIBUTE);
        e1.set_target_range(duel, 0, LOCATION_MZONE);
        e1.set_value(duel, (POS_FACEUP | POS_FACEDOWN) as i32);
        e1.set_reset(duel, RESET_CHAIN);
        c.register_effect_full(duel, e1, true);
        let res = c.can_summon_or_set(duel, true, None, 1);
        e1.reset(duel);
        res
    })
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|sc, duel| sumfilter(sc, duel, c), a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let Some(tc) = duel
        .select_matching_card(tp, |sc, duel| sumfilter(sc, duel, c), tp, LOCATION_HAND, 0, 1, 1, None)?
        .first()
    else {
        return Ok(());
    };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_ADD_EXTRA_TRIBUTE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, (POS_FACEUP | POS_FACEDOWN) as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect_full(duel, e1, true);
    duel.summon_or_set(tp, tc, true, None, 1)?;
    let e2 = duel.create_effect(c);
    e2.set_description(duel, 3303);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e2.set_value(duel, 1);
    e2.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD) | RESET_PHASE | PHASE_END);
    tc.register_effect_full(duel, e2, true);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UNRELEASABLE_NONSUM);
    tc.register_effect_full(duel, e3, true);
    Ok(())
}

fn opsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let tp = 1 - a.tp;
    if duel.is_existing_matching_card(|sc, duel| sumfilter(sc, duel, c), tp, LOCATION_HAND, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        activate(duel, &Args { tp, ..*a })?;
    }
    Ok(())
}
