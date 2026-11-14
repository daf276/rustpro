// Speedroid Den-Den Daiko Duke
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 59640711;

pub const LISTED_SERIES: &[u16] = &[SET_SPEEDROID];
pub const LISTED_NAMES: &[u32] = &[ID];

// Speedroid Darumaknocker (a Tuner) in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (85482105, HAND), (85482105, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SPEEDROID])
        && c.is_type(duel, TYPE_TUNER)
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(
        duel,
        a,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        LOCATION_HAND | LOCATION_GRAVE,
        POS_FACEUP,
    )
}
