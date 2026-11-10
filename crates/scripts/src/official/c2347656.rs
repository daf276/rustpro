// Lovely Labrynth of the Silver Castle
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 2347656;

// On the field, with Normal Traps (Trap Hole in the Graveyard to Set, Mirror Force Set) to make a monster leave the field, and the
// opponent's monsters and hand to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (4206964, GRAVE),
    (44095762, SZONE),
    (44095762, SZONE),
    (4206964, SZONE),
    (1225009, MZONE | OPPONENT),
    (1225009, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, chainop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, fgtg);
    e2.set_operation(duel, fgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn chainop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if a.rp == a.tp && re.get_handler(duel).is_normal_trap(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        duel.set_chain_limit(|duel, e, rp, tp, _| rp == tp || !e.is_monster_effect(duel), 0);
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && c.is_ssetable(duel)
}

fn fgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn fgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.sset(tp, tc)? > 0
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_condition(duel, triggercon);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn triggercon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FIEND));
    !duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_EFFECT)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.rp == a.tp
        && a.re
            .is_some_and(|re| re.is_trap_effect(duel) && re.get_handler(duel).get_original_type(duel) == TYPE_TRAP)
        && a.eg.is_some_and(|eg| eg.is_exists(duel, cfilter, 1, None))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_ONFIELD | LOCATION_HAND, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, (LOCATION_ONFIELD | LOCATION_HAND) as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    let g2 = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_HAND, None);
    let (b1, b2) = (!g1.is_empty(), !g2.is_empty());
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    let mut sg = CardSet::default();
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        sg = g1.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(sg);
    } else if op == Some(2) {
        sg = g2.random_select(duel, tp, 1);
    }
    if !sg.is_empty() {
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}
