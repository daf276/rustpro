// Theorealize
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 48739627;

pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE];

// Medius the Pure in the Deck to send, a copy in the Graveyard and a monster on the field to change.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_MEDIUS_THE_PURE, DECK),
    (CARD_MEDIUS_THE_PURE, DECK),
    (ID, GRAVE),
    (13597785, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, |duel, a, chk, _| {
        Ok(chk != 0 || duel.get_field_group_count(a.tp, 0, LOCATION_DECK) > 0)
    });
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, chngtg);
    e2.set_operation(duel, chngop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MEDIUS_THE_PURE) && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_decktop_group(1 - tp, 1);
    if let Some(tc) = g.get_first() {
        duel.confirm_cards(tp, g)?;
        duel.hint(HINT_SELECTMSG, tp, 0);
        if duel.select_option(tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])? == 1 {
            duel.move_sequence(tc, 1)?;
        }
    }
    Ok(())
}

fn chngtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tc = duel
        .select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(false) };
    let race = duel.announce_race(tp, 1, RACE_ALL)?;
    let attr = if tc.is_race_except(duel, race) {
        duel.announce_attribute(tp, 1, ATTRIBUTE_ALL)?
    } else {
        tc.announce_another_attribute(duel, tp)?
    };
    e.set_labels(duel, &[race as i64, i64::from(attr)]);
    Ok(false)
}

fn chngop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let &[race, attr] = e.get_labels(duel) else { return Ok(()) };
    if tc.is_race_except(duel, race as u64) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_RACE);
        e1.set_value(duel, race as i32);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e1);
    }
    if tc.is_attribute_except(duel, attr as u32) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e2.set_value(duel, attr as i32);
        e2.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
