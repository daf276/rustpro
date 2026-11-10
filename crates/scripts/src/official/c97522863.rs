// The Man with the Mark
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 97522863;

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS];
// Three copies in the hand to Normal Summon, with Temple of the Kings in the Deck to find.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (CARD_TEMPLE_OF_THE_KINGS, DECK),
    (CARD_TEMPLE_OF_THE_KINGS, DECK),
    (CARD_TEMPLE_OF_THE_KINGS, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtgtg);
    e1.set_operation(duel, thtgop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS));
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_ONFIELD, 0, 1, None)
    });
    e3.set_target_card(duel, |duel, e, c| c == e.get_handler(duel) || c.is_set_card(duel, &[SET_APOPHIS]));
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e4.set_target_range(duel, LOCATION_ONFIELD, 0);
    e4.set_target_card(duel, |duel, e, c| {
        c == e.get_handler(duel) || (c.is_set_card(duel, &[SET_APOPHIS]) && c.is_monster(duel)) || c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS)
    });
    c.register_effect(duel, e4);
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS) || (c.is_spell_trap(duel) && c.lists_code(duel, CARD_TEMPLE_OF_THE_KINGS)))
        && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.send_to_grave(tc, REASON_EFFECT).map(drop),
            574,
        )?;
    }
    Ok(())
}
