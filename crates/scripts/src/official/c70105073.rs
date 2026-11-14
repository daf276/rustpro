// Ritual of the Matador
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 70105073;
const MATADOR: u32 = 7622360;

pub const LISTED_NAMES: &[u32] = &[MATADOR];
pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];
pub const FIT_MONSTER: &[u32] = &[MATADOR];

struct Params;

impl RitualCode for Params {
    const LV: Option<u32> = Some(6);
    const CODES: &'static [u32] = FIT_MONSTER;
}

// Archfiend Matador with Level 4 Archfiend Soldiers in hand as the Tribute and to Special Summon, and a copy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(MATADOR, HAND), (49881766, HAND), (49881766, HAND), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(MATADOR), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater_code::<Params>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
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
    !c.is_ritual_monster(duel) && c.is_set_card(duel, &[SET_ARCHFIEND]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}
