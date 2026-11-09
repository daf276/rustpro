// Exchange
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT};

const ID: u32 = 5556668;

// Psychic Kappa, a Normal Monster, in the hand to give, and one in the opponent's hand to take.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (7892180, HAND), (7892180, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk != 0
        || (duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_HAND, 0, 1, Some(a.e.get_handler(duel).into()))))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_field_group(tp, LOCATION_HAND, 0);
    let g2 = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g1.is_empty() || g2.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, g2)?;
    duel.confirm_cards(1 - tp, g1)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let ag1 = g2.select(duel, tp, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
    let ag2 = g1.select(duel, 1 - tp, 1, 1, None)?;
    duel.send_to_hand(ag1, tp, REASON_EFFECT)?;
    duel.send_to_hand(ag2, 1 - tp, REASON_EFFECT)?;
    Ok(())
}
