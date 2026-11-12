// Mind Master
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 96782886;

// On the field with Psychic Normal Monsters (Chosen by the World Chalice, Stinger the Electric Expeller) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (22916281, DECK), (22916281, DECK), (57649113, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::pay_lp::<800>, spcost]));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let filter = |c: CardId, duel: &Duel| c.is_race(duel, RACE_PSYCHIC);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, Some(&aux::release_check_mmz), None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, Some(&aux::release_check_mmz), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_race(duel, RACE_PSYCHIC)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_ATTACK)
}
