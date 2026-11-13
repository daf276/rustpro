// True Exodia
use engine::constants::MSG_WIN;

use crate::prelude::*;
use crate::testing::{Check, MZONE};

// The four limbs (Normal Monsters) on the field with it.
pub const SUPPORT: &[(u32, u16)] = &[(7902349, MZONE), (8124921, MZONE), (44519536, MZONE), (70903634, MZONE)];
// Every duel ends in MSG_WIN; the ones this card wins carry reason 0x21 (scenarios 5 and 10).
pub const CHECKS: &[Check] = &[Check::Message(MSG_WIN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(
        duel,
        EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_ADJUST);
    e1.set_operation(duel, loseop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_NORMAL) && c.is_set_card(duel, &[SET_FORBIDDEN_ONE])
}

fn loseop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(CardSet::from(c)));
    if g.filter_count(duel, filter, None) == g.len() && g.len() == 4 && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == 4
    {
        duel.win(1 - c.get_controler(duel), WIN_REASON_TRUE_EXODIA);
    }
    Ok(())
}
