// Karakuri Steel Shogun mdl 00X "Bureido"
use super::c23874409::{Procedure, spcon, spop, sptg};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 66976526;

pub const LISTED_SERIES: &[u16] = &[SET_KARAKURI];
// In the Extra Deck, Synchro Summoned with Genex Controller (a Level 3 Tuner) and Shapesnatch (a Level 5 Machine), with Karakuri
// Gama in the Deck to summon and Karakuri monsters on the field to change position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68505803, MZONE),
    (68505803, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (75690317, DECK),
    (75690317, DECK),
    (75690317, DECK),
    (92300891, MZONE),
    (92300891, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_CHANGE_POS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let (np, pp) = (c.get_position(duel), c.get_previous_position(duel));
    c.is_set_card(duel, &[SET_KARAKURI])
        && c.is_controler(duel, tp)
        && ((pp == POS_FACEUP_ATTACK && np == POS_FACEUP_DEFENSE) || (pp == POS_FACEUP_DEFENSE && np == POS_FACEUP_ATTACK))
}

pub fn drcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(!a.e.get_handler(duel).is_status(duel, STATUS_CHAINING) && duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}
