// Doublebyte Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 23971061;
const HONEYBOT: u32 = 34472920;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Honeybots (Link Monsters).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (HONEYBOT, MZONE), (HONEYBOT, MZONE), (HONEYBOT, MZONE)];
// No activated effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_link_monster(duel))));
    c.register_effect(duel, e3);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let atk: u32 = c.get_material(duel).iter().map(|tc| tc.get_link(duel)).sum();
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, atk as i32 * 300);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel) && te.get_owner(duel) != e.get_owner(duel) && !te.is_active_type(duel, TYPE_LINK)
}
