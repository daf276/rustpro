// Road Warrior
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 2322421;
const ROAD_SYNCHRON: u32 = 71971554;

// In the Extra Deck, Synchro Summoned with Road Synchron and Doron (Level 2 Normal Monsters) on the field, and Level 2 or lower
// Machines (Satellite Synchron, Unknown Synchron) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ROAD_SYNCHRON, MZONE),
    (ROAD_SYNCHRON, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (57458399, DECK),
    (57458399, DECK),
    (15310033, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        tfilter(c, duel, sc, sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[ROAD_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[ROAD_SYNCHRON];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn tfilter(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_summon_code(duel, Some(sc), sumtype, tp, &[ROAD_SYNCHRON]) || c.is_has_effect(duel, 20932152).is_some()
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 2) && c.is_race(duel, RACE_WARRIOR | RACE_MACHINE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
