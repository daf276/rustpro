// Santa Claws
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 46565218;

struct Procedure;

impl aux::LavaProcedure for Procedure {
    const REQUIRED: usize = 1;
}

// A copy in the hand and a Sangan in the opponent's monster zone to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::add_lava_procedure::<Procedure>(duel, c, POS_FACEUP_DEFENSE, 1, None);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
    });
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_OATH);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    e1.set_reset(
        duel,
        RESET_EVENT | (RESETS_STANDARD_PHASE_END & !(RESET_LEAVE | RESET_TEMP_REMOVE | RESET_TURN_SET)),
    );
    c.register_effect(duel, e1);
    Ok(())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
