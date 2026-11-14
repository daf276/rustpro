// Spiritual Beast Cannahawk
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 49885567;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];
// On the field with Ritual Beast monsters in the Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (14513016, DECK), (88123329, DECK), (88123329, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_REMOVE)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_range(duel, LOCATION_REMOVED);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_count_limit(duel, 1, 0);
        e1.set_reset_full(
            duel,
            RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
            2,
        );
        e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
        e1.set_operation(duel, thop);
        e1.set_label(duel, 0);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = e.get_label(duel);
    let c = e.get_handler(duel);
    c.set_turn_counter(duel, (ct + 1) as u16);
    if ct == 1 {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    } else {
        e.set_label(duel, 1);
    }
    Ok(())
}
