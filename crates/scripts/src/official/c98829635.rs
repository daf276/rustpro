// Forbidden Crown
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 98829635;

// Sangan as the face-up monster to apply the effects to.
pub const SUPPORT: &[(u32, u16)] = &[(26202165, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.has_flag_effect(duel, ID)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_chain_limit(|duel, te, _, _, _| !te.is_monster_effect(duel), 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(sc);
    let c = a.e.get_handler(duel);
    sc.register_flag_effect_full(
        duel,
        ID,
        RESETS_STANDARD_PHASE_END,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 1),
    );
    sc.negate_effects(duel, c, RESETS_STANDARD_PHASE_END, false);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    sc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    sc.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    sc.register_effect(duel, e3);
    let e4 = e1.clone_effect(duel);
    e4.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e4.set_value(
        duel,
        Value::Effect(|duel, e, re| re.is_activated(duel) && e.get_handler(duel) != re.get_owner(duel)),
    );
    sc.register_effect(duel, e4);
    let e5 = e2.clone_effect(duel);
    e5.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    sc.register_effect(duel, e5);
    let e6 = e2.clone_effect(duel);
    e6.set_code(duel, EFFECT_UNRELEASABLE_NONSUM);
    sc.register_effect(duel, e6);
    let e7 = e1.clone_effect(duel);
    e7.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e7.set_value(
        duel,
        Value::Material(|_, _, _, sumtype, _| {
            let allowed = [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK];
            aux::cannotmatfilter(&allowed.map(u64::from), sumtype)
        }),
    );
    sc.register_effect(duel, e7);
    Ok(())
}
