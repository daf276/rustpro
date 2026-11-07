// Non-Fusion Area
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 27581098;

// Polymerization with Fallen of Albaz and Petit Angel, to attempt the Fusion Summon of Albion the Branded Dragon in the Elfnote Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (24094653, HAND), (68468459, HAND), (38142739, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_target_spsummon(duel, |_, _, _, _, sumtp, _, _, _| sumtp & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION);
    c.register_effect(duel, e2);
}
