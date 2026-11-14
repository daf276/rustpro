// Bingo Machine, Go!!!
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 93437091;

pub const LISTED_SERIES: &[u16] = &[SET_BLUE_EYES];
pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON, 23995346];

// Blue-Eyes White Dragon, a Normal Monster, in the Deck: three copies to pick from.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_monster(duel))
        || ((c.lists_code(duel, CARD_BLUEEYES_W_DRAGON) || c.lists_code(duel, 23995346)) && c.is_spell_trap(duel) && !c.is_code(duel, ID)))
        && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 3, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if g.len() >= 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let sg = g.select(duel, tp, 3, 3, None)?;
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_deck(tp)?;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
        if let Some(tc) = sg.select(duel, 1 - tp, 1, 1, None)?.get_first() {
            duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
