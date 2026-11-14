// Simul Archfiends
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 94423983;

const TYPE_RITUAL_FUSION_SYNCHRO_XYZ_LINK: u32 = TYPE_RITUAL | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.get_flag_effect(0, ID) == 0)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 1);
    e1.set_target_spsummon(duel, sumlimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let filter = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_RITUAL_FUSION_SYNCHRO_XYZ_LINK));
    let g = duel.get_matching_group(filter, tp, LOCATION_MZONE, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    for typ in [TYPE_RITUAL, TYPE_FUSION, TYPE_SYNCHRO, TYPE_XYZ, TYPE_LINK] {
        if g.is_exists(duel, |c, duel| c.is_type(duel, typ), 2, None) {
            return Ok(());
        }
    }
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn sumlimit(duel: &Duel, _: EffectId, c: CardId, sump: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    let g = duel.get_matching_group(CardId::is_faceup, sump, LOCATION_MZONE, 0, None);
    let mtypes = g.get_bitwise_or(duel, |c, duel| u64::from(c.get_type(duel))) as u32 & TYPE_RITUAL_FUSION_SYNCHRO_XYZ_LINK;
    c.is_type(duel, mtypes)
}
