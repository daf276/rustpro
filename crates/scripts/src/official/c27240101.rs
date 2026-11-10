// Kikinagashi Fucho
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 27240101;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned from two Level 1 Normal Monsters (Key Mace, White Duston).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (1929294, MZONE),
    (3557275, MZONE),
    (3557275, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e2.set_operation(duel, indop);
    e2.set_hint_timing(duel, 0, TIMING_BATTLE_START);
    c.register_effect(duel, e2);
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.get_owner(duel) != e.get_owner(duel)
}

fn indop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3000);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_description(duel, 3210);
        e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
        c.register_effect(duel, e2);
    }
    Ok(())
}
