// Soul Unseal
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 90506641;

// Copies in the hand, with Normal Monsters (Water Omotics) on the field and in the Deck, and an opponent's monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (2483611, MZONE),
    (2483611, MZONE),
    (2483611, DECK),
    (2483611, DECK),
    (2483611, DECK),
    (2483611, DECK),
    (2483611, DECK),
    (2483611, DECK),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_NORMAL)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(1 - a.tp, LOCATION_MZONE, 0) > 0 && duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && (c.is_able_to_hand(duel) || c.is_able_to_remove(duel))
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.filter_count(duel, CardId::is_able_to_hand, None) > 0 && sg.filter_count(duel, CardId::is_able_to_remove, None) > 3,
        false,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 5, 5, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 4, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() < 5 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sg = aux::select_unselect_group(duel, g, 5, 5, &rescon, tp, HINTMSG_ATOHAND)?;
    if sg.len() != 5 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sc = if sg.filter_count(duel, CardId::is_able_to_hand, None) > 0 {
        sg.filter_select(duel, tp, CardId::is_able_to_hand, 1, 1, None)?
    } else {
        sg.select(duel, tp, 1, 1, None)?
    };
    let sg = sg.difference(sc);
    if !sg.is_empty() && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? == 0 {
        duel.send_to_grave(sc, REASON_RULE)?;
    }
    duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
