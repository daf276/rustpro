// Exodia the Forbidden One
use engine::constants::MSG_WIN;

use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 33396948;

pub const LISTED_NAMES: &[u32] = &[8124921, 44519536, 70903634, 7902349];
// The four limbs (Normal Monsters) in the hand with it, so the first draw wins.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (8124921, HAND), (44519536, HAND), (70903634, HAND), (7902349, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_WIN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn check(duel: &Duel, g: CardSet) -> bool {
    [8124921, 44519536, 70903634, 7902349, ID]
        .into_iter()
        .all(|code| g.is_exists(duel, |c, duel| c.get_code(duel) == code, 1, None))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_field_group(tp, LOCATION_HAND, 0);
    let g2 = duel.get_field_group(tp, 0, LOCATION_HAND);
    match (check(duel, g1), check(duel, g2)) {
        (true, false) => {
            duel.confirm_cards(1 - tp, g1)?;
            duel.win(tp, WIN_REASON_EXODIA);
        }
        (false, true) => {
            duel.confirm_cards(tp, g2)?;
            duel.win(1 - tp, WIN_REASON_EXODIA);
        }
        (true, true) => {
            duel.confirm_cards(1 - tp, g1)?;
            duel.confirm_cards(tp, g2)?;
            duel.win(PLAYER_NONE, WIN_REASON_EXODIA);
        }
        (false, false) => {}
    }
    Ok(())
}
