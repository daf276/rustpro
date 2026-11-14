// FaSolfachord Fancia
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, PZONE};

const ID: u32 = 54100561;

pub const LISTED_SERIES: &[u16] = &[SET_SOLFACHORD];
// On the field with DoSolfachord Cutia (Scale 1) in the Pendulum Zone and in the Deck to send to the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (55226153, PZONE), (55226153, DECK), (55226153, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DISABLE_SPSUMMON);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_SET_AVAILABLE);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_pendulum_summoned(duel) && c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_type(duel, TYPE_PENDULUM)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tetg);
    e2.set_operation(duel, teop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_odd_scale),
            a.e.get_handler_player(duel),
            LOCATION_PZONE,
            0,
            1,
            None,
        )
    });
    e3.set_target(duel, reptg);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(filter(c, duel, e.get_handler_player(duel)))),
    );
    e3.set_operation(duel, |duel, a| {
        duel.destroy(a.e.get_handler(duel), REASON_EFFECT | REASON_REPLACE)?;
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn tefilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && c.is_set_card(duel, &[SET_SOLFACHORD]) && !c.is_code(duel, ID) && !c.is_forbidden(duel)
}

fn tetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tefilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn teop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(tp, tefilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_extra_p(g, tp, REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_set_card(duel, &[SET_SOLFACHORD])
        && c.is_reason(duel, REASON_BATTLE)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| filter(c, duel, tp), 1, None)
            && c.is_destructable_full(duel, a.e)
            && !c.is_status(duel, STATUS_DESTROY_CONFIRMED));
    }
    duel.select_effect_yes_no(tp, c)
}
