// Lava Golem
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 102380;

struct Procedure;

impl aux::LavaProcedure for Procedure {
    const REQUIRED: usize = 2;
}

// Two Sangan in the opponent's monster zone to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    aux::add_lava_procedure::<Procedure>(duel, c, POS_FACEUP, 0, Some(aux::stringid(ID, 0)));
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EFFECT_SPSUMMON_COST);
    e3.set_cost_condition(duel, |duel, _, _, tp, _| duel.get_activity_count(tp, ACTIVITY_NORMALSUMMON) == 0);
    e3.set_operation(duel, spcostop);
    c.register_effect(duel, e3);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 1000);
    Ok(false)
}

fn spcostop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_MSET);
    duel.register_effect(e2, tp);
    Ok(())
}
