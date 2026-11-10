// Zombie World
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 4064256;

// In the hand, with monsters of other races on both sides that it changes to Zombies.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (6103114, MZONE),
    (6103114, MZONE),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE | LOCATION_GRAVE, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_code(duel, EFFECT_CHANGE_RACE);
    e2.set_value(duel, RACE_ZOMBIE as i32);
    e2.set_target_card(duel, tg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_code(duel, EFFECT_CANNOT_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 1);
    e3.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        sumtype & SUMMON_TYPE_TRIBUTE == SUMMON_TYPE_TRIBUTE && !c.is_race(duel, RACE_ZOMBIE)
    });
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_CANNOT_MSET);
    c.register_effect(duel, e4);
    // Only read by c5288597, which is not ported, so its value is the race itself.
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, ID);
    e5.set_range(duel, LOCATION_FZONE);
    e5.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e5.set_target_range(duel, 1, 1);
    e5.set_value(duel, RACE_ZOMBIE as i32);
    c.register_effect(duel, e5);
}

// The Lua registers and resets flag 1 around the lookup to stop it recursing, which no Necro Valley effect's condition
// does here; the flag only takes up a field id.
fn tg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    duel.skip_field_id();
    let effects = if c.is_location(duel, LOCATION_MZONE) {
        duel.get_player_effect(c.get_controler(duel), EFFECT_NECRO_VALLEY)
    } else {
        c.get_card_effect(duel, EFFECT_NECRO_VALLEY)
    };
    for te in effects.iter() {
        let op = te.get_operation(duel);
        unported!(duel, op.is_some());
        if op.is_none() {
            return false;
        }
    }
    true
}
