// Number 81: Superdreadnought Rail Cannon Super Dora
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 49032236;
pub const XYZ_NUMBER: u32 = 81;

// In the Extra Deck, Xyz Summoned with Chaos Angel and Ruddy Rose Dragon (both Level 10; there is no Level 10 Normal Monster).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (22850702, MZONE), (40139997, MZONE), (42410161, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_hint_timing(
        duel,
        TIMING_STANDBY_PHASE | TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_END_PHASE,
    );
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3100);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(duel, Value::Effect(efilter));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    let c = e.get_handler(duel);
    c != re.get_owner(duel) && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}
