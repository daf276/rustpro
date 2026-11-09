// Fire Formation - Tenki
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 57103969;

// In the hand to activate, with Battle Ox (a Level 4 Beast-Warrior Normal Monster) in the Deck to search and on the field to
// attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (5053103, DECK), (5053103, DECK), (5053103, DECK), (5053103, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_BEASTWARRIOR));
    e2.set_value(duel, 100);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_BEASTWARRIOR) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
