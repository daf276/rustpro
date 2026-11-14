// T.G. Warwolf
use super::c1315120::regop;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 293542;

pub const LISTED_SERIES: &[u16] = &[SET_TG];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand to Special Summon after Sangan is Special Summoned, as the hand Synchro material T.G. Cyber Magician grants for T.G.
// Hyper Librarian (with Key Mace for Level 5), and on the field to be destroyed by the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (26202165, HAND),
    (64910482, MZONE),
    (64910482, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (90953320, EXTRA),
    (90953320, EXTRA),
    (90953320, EXTRA),
    (94350039, DECK),
    (94350039, DECK),
    (12580477, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::MaterialFrom(HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop::<ID, 1>);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| c.is_faceup(duel) && c.is_level_below(duel, 4), 1, None)
}
