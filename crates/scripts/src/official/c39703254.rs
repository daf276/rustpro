// Inmato
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 39703254;

// On the field and in the hand, with Mushroom Man (a Plant Normal Monster) as the target of the opponent's Mage Power.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (14181608, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.rp == a.tp || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) || !re.is_spell_trap_effect(duel) {
        return false;
    }
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    let Some(tg) = g.filter(|g| g.len() == 1).and_then(|g| g.get_first()) else {
        return false;
    };
    tg != a.e.get_handler(duel) && tg.is_faceup(duel) && tg.is_race(duel, RACE_PLANT)
}
