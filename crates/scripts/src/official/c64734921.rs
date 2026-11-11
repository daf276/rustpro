// The Agent of Creation - Venus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 64734921;
const SHINE_BALL: u32 = 39552864;

pub const LISTED_NAMES: &[u32] = &[SHINE_BALL];
// On the field with Mystical Shine Ball in the Deck and hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (SHINE_BALL, DECK), (SHINE_BALL, DECK), (SHINE_BALL, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::pay_lp::<500>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, SHINE_BALL) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK | LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_HAND,
        POS_FACEUP,
    )
}
