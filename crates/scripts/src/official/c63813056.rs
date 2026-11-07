// Xtra HERO Dread Decimator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 63813056;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_HERO], Some(lc), sumtype, tp)
    }
}

// No effect resolves, so its Link Summon is checked: in the Extra Deck, with two Elemental HERO Stratos on the field
// and HERO monsters in the GY for its ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (40044918, MZONE),
    (40044918, MZONE),
    (9411399, GRAVE),
    (16605586, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, atktg);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}

fn atktg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let handler = e.get_handler(duel);
    (handler.get_linked_group(duel).is_contains(c) || c == handler) && c.is_set_card(duel, &[SET_HERO]) && c.is_faceup(duel)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERO]) && c.is_monster(duel)
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = duel.get_matching_group(atkfilter, e.get_handler_player(duel), LOCATION_GRAVE, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32 * 100
}
