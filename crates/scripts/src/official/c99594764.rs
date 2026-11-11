// Dragunity Corsesca
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 99594764;

// Dragunity Falchion in hand to equip a copy from the Deck to Dragunity Knight - Luin, which battles Elfnote Lucina of the opponent's;
// the Deck holds Dragunity monsters to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (8039562, HAND),
    (8039562, HAND),
    (8039562, HAND),
    (12496261, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    eg.len() == 1 && eg.get_first() == a.e.get_handler(duel).get_equip_target(duel)
}

fn filter(c: CardId, duel: &Duel, race: u64, att: u32) -> bool {
    c.is_race(duel, race) && c.is_attribute(duel, att) && c.get_level(duel) <= 4 && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(eqc) = a.e.get_handler(duel).get_equip_target(duel) else {
        return Ok(false);
    };
    let (race, att) = (eqc.get_race(duel), eqc.get_attribute(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, race, att), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(eqc) = a.e.get_handler(duel).get_equip_target(duel) else {
        return Ok(());
    };
    let (race, att) = (eqc.get_race(duel), eqc.get_attribute(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, race, att), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
