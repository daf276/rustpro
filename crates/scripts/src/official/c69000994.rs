// Fire King Avatar Barong
use super::c96594609::{spcon, sptg};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 69000994;
const FIRE_KING_ISLAND: u32 = 57554544;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_KING];
// Copies in hand and on the field, Fire King Island and Fire King Courtier Ulcanix to destroy a Fire King, Fire Kings in the
// Deck to add, and the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (FIRE_KING_ISLAND, HAND),
    (44455560, MZONE),
    (44455560, DECK),
    (90681088, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, threg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, |duel, a| stock::search(duel, a, thfilter, LOCATION_DECK));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn threg(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if a.r & (REASON_DESTROY | REASON_EFFECT) != REASON_DESTROY | REASON_EFFECT {
        return Ok(());
    }
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY;
    if duel.is_phase(PHASE_STANDBY) {
        let turn = duel.get_turn_count();
        e.set_label(duel, turn.into());
        c.register_flag_effect(duel, ID, reset, 0, 2);
    } else {
        e.set_label(duel, 0);
        c.register_flag_effect(duel, ID, reset, 0, 1);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel).map_or(0, |e2| e2.get_label(duel)) != i64::from(duel.get_turn_count())
        && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FIRE_KING]) && c.get_code(duel) != ID && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    a.e.get_handler(duel).reset_flag_effect(duel, ID);
    Ok(false)
}
