// Blizzard Princess
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 28348537;

struct Procedure;

impl NormalProcedure for Procedure {
    const NS: bool = true;
    const OPT: bool = true;
    const MIN: i32 = 1;
    const MAX: i32 = 1;
    const DESC: Option<u64> = Some(aux::stringid(ID, 0));

    fn f(c: CardId, duel: &Duel, tp: u8) -> bool {
        c.is_race(duel, RACE_SPELLCASTER) && (c.is_controler(duel, tp) || c.is_faceup(duel))
    }
}

// In the hand twice, to be Tribute Summoned with a Spellcaster on the field: Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (13597785, MZONE)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Normal::add_summon_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_operation(duel, actlimit);
    c.register_effect(duel, e2);
}

fn actlimit(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, te, _| te.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}
