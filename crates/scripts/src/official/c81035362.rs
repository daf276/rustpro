// Fiendish Rhino Warrior
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 81035362;

// Copies on the field, destroyed by the opponent's Dark Hole to send Fiend Normal Monsters (Bio Plant, The Drdek) from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (7670542, MZONE),
    (7670542, DECK),
    (8944575, DECK),
    (8944575, DECK),
    (53129443, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, indtg);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn indtg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_race(duel, RACE_FIEND) && !c.is_code(duel, ID)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(a.tp, tgfilter, a.tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
