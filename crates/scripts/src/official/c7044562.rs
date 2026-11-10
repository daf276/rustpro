// Card of the Soul
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 7044562;

// Hamon, Lord of Striking Thunder (4000 ATK and DEF) in the Deck matches the starting Life Points of 8000.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (32491822, DECK), (32491822, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, lp: i32) -> bool {
    c.get_attack(duel) >= 0 && c.get_defense(duel) >= 0 && c.get_attack(duel) + c.get_defense(duel) == lp && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_DECK, 0);
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, g)?;
    let lp = duel.get_lp(tp);
    let filter = |c, duel: &Duel| filter(c, duel, lp);
    if g.is_exists(duel, filter, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.filter_select(duel, tp, filter, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_hand(tp)?;
    }
    duel.shuffle_deck(tp)?;
    Ok(())
}
