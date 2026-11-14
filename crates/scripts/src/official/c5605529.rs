// Vaalmonica Scelta
use super::c42193638::select_option;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 5605529;

pub const LISTED_SERIES: &[u16] = &[SET_VAALMONICA];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with Angello and Dimonno in the Pendulum Zone, a card in the hand to return and Vaalmonica Spells in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (3048768, PZONE),
    (30432463, PZONE),
    (65496951, DECK),
    (42193638, DECK),
    (78598237, DECK),
    (78598237, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_RECOVER + CATEGORY_TODECK + CATEGORY_DRAW + CATEGORY_DAMAGE + CATEGORY_TOHAND + CATEGORY_SEARCH,
    );
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VAALMONICA]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let op = select_option(duel, a, ID)?;
    if op == Some(1) {
        let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATION_HAND, 0, None);
        if duel.recover(tp, 500, REASON_EFFECT)? > 0
            && !g.is_empty()
            && duel.is_player_can_draw(tp, 2)
            && duel.select_yes_no(tp, aux::stringid(ID, 5))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
                return Ok(());
            };
            duel.break_effect()?;
            if !(duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_DECK)) {
                return Ok(());
            }
            duel.draw(tp, 2, REASON_EFFECT)?;
        }
    } else if op == Some(2) {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        if duel.damage(tp, 500, REASON_EFFECT)? > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 6))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
