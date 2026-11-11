// Iron Blacksmith Kotetsu
use crate::prelude::*;
use crate::testing::{DECK, MZONE, OPPONENT};

const ID: u32 = 73431236;

// Face-down copies flipped by the opponent's attackers, with Equip Spells (Axe of Despair, Mystical Moon, Salamandra)
// in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (40619825, DECK),
    (36607978, DECK),
    (32268901, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
