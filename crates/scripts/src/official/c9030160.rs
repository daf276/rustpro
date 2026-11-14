// Dark Contract with the Eternal Darkness
use crate::prelude::*;
use crate::testing::{Check, HAND, PZONE, SZONE};

const ID: u32 = 9030160;

pub const LISTED_SERIES: &[u16] = &[SET_DD];
// Face-up and in the hand, with D/D Pendulums in both Pendulum Zones for its continuous effects.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, HAND), (11609969, PZONE), (42382265, PZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_value(duel, Value::EffectPlayer(evalue));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_condition(duel, condition);
    e3.set_value(duel, Value::OptionalCard(sumlimit));
    c.register_effect(duel, e3);
    let e5 = e3.clone_effect(duel);
    e5.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e5.set_value(
        duel,
        Value::Material(|duel, e, c, sumtype, tp| {
            let allowed = [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ];
            matlimit(duel, e, c, tp) && aux::cannotmatfilter(&allowed.map(u64::from), sumtype)
        }),
    );
    c.register_effect(duel, e5);
    let e8 = duel.create_effect(c);
    e8.set_description(duel, aux::stringid(ID, 0));
    e8.set_category(duel, CATEGORY_DAMAGE);
    e8.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e8.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e8.set_range(duel, LOCATION_SZONE);
    e8.set_count_limit(duel, 1, 0);
    e8.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e8.set_target(duel, super::c46372010::damtg::<1000>);
    e8.set_operation(duel, super::c46372010::damop);
    c.register_effect(duel, e8);
}

fn matlimit(duel: &Duel, e: EffectId, c: Option<CardId>, tp: u8) -> bool {
    unported!(duel, tp == PLAYER_NONE && c.is_none());
    let tp = match c {
        Some(c) if tp == PLAYER_NONE => c.get_controler(duel),
        _ => tp,
    };
    e.get_handler_player(duel) == 1 - tp
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_set_card(duel, &[SET_DD]),
        a.e.get_handler_player(duel),
        LOCATION_PZONE,
        0,
        2,
        None,
    )
}

fn sumlimit(duel: &Duel, e: EffectId, c: Option<CardId>) -> i32 {
    c.is_some_and(|c| !c.is_controler(duel, e.get_handler_player(duel))).into()
}

fn evalue(duel: &Duel, e: EffectId, re: EffectId, rp: u8) -> bool {
    re.is_spell_trap_effect(duel) && rp == 1 - e.get_handler_player(duel)
}
