// Stand Up Centur-Ion!
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 41371602;

pub const LISTED_SERIES: &[u16] = &[SET_CENTUR_ION];
// In the hand with Primera, Trudea and Mystical Elf (Level 12 together) on the field to Synchro Summon Legatia or Auxila, and
// Primera and Trudea in the Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (15005145, MZONE),
    (42493140, MZONE),
    (15025844, MZONE),
    (15982593, EXTRA),
    (71858682, EXTRA),
    (15005145, DECK),
    (42493140, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, reg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, indcon);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e3.set_cost(duel, tfcost);
    e3.set_target(duel, tftg);
    e3.set_operation(duel, tfop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, syntg);
    e4.set_operation(duel, synop);
    c.register_effect(duel, e4);
}

fn reg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn indfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_original_type(duel, TYPE_MONSTER) && c.is_set_card(duel, &[SET_CENTUR_ION])
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(indfilter, a.e.get_handler_player(duel), LOCATION_ONFIELD, 0, 1, None)
}

fn tfcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_monster(duel) && !c.is_forbidden(duel)
}

fn tftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.is_existing_matching_card(plfilter, tp, LOCATION_DECK, 0, 1, None))
}

fn tfop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_TRAP | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn syncheck(duel: &Duel, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_CENTUR_ION]), 1, None)
}

fn syntg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        duel.set_synchro_check_additional(Some(syncheck));
        let res = duel.is_existing_matching_card(|c, duel| c.is_synchro_summonable(duel, None, None), tp, LOCATION_EXTRA, 0, 1, None);
        duel.set_synchro_check_additional(None);
        return Ok(res);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.set_synchro_check_additional(Some(syncheck));
    let g = duel.get_matching_group(|c, duel| c.is_synchro_summonable(duel, None, None), tp, LOCATION_EXTRA, 0, None);
    if g.is_empty() {
        duel.set_synchro_check_additional(None);
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.select(duel, tp, 1, 1, None)?;
    if let Some(sc) = sg.get_first() {
        duel.synchro_summon(tp, sc, None, None)?;
    }
    Ok(())
}
