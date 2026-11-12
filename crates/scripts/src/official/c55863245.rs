// Tatsunoko
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 55863245;

// Synchro Summoned from Water Spirit (Level 1 Tuner) and Mushroom Man (Level 2), then Armory Arm (Level 4) with Dark Plant (Level 1) from the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (29071332, EXTRA),
    (29071332, EXTRA),
    (13193642, HAND),
    (13193642, HAND),
];
// Its effects form no chain link.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];
const HAND_CHECK: u32 = EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_condition(duel, syncon);
    e3.set_code(duel, EFFECT_HAND_SYNCHRO);
    e3.set_label(duel, ID.into());
    e3.set_value(duel, Value::HandSynchro(synval));
    c.register_effect(duel, e3);
}

fn syncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SYNCHRO
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel) && te.get_owner(duel) != e.get_owner(duel)
}

fn synval(duel: &mut Duel, e: EffectId, c: CardId, _: CardId) -> bool {
    if !c.is_location(duel, LOCATION_HAND) {
        return false;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, HAND_CHECK);
    e1.set_label(duel, ID.into());
    e1.set_synchro_check_target(duel, synchktg);
    c.register_effect(duel, e1);
    true
}

fn chk(c: CardId, duel: &Duel) -> bool {
    c.is_has_effect(duel, HAND_CHECK).is_some()
        && c.get_card_effect(duel, HAND_CHECK)
            .iter()
            .all(|&e| e.get_label(duel) == i64::from(ID))
}

fn chk2(c: CardId, duel: &Duel) -> bool {
    c.is_has_effect(duel, EFFECT_HAND_SYNCHRO).is_some()
        && c.is_has_effect(duel, HAND_CHECK).is_none()
        && c.get_card_effect(duel, EFFECT_HAND_SYNCHRO)
            .iter()
            .any(|&e| e.get_label(duel) == i64::from(ID))
}

fn synchktg(
    duel: &Duel,
    _: EffectId,
    c: Option<CardId>,
    sg: CardSet,
    tg: CardSet,
    ntg: CardSet,
    _: CardSet,
    _: CardSet,
) -> (bool, CardSet, CardSet) {
    let none = CardSet::default();
    let Some(c) = c else { return (true, none, none) };
    let others = Some(c.into());
    if sg.is_exists(duel, chk, 1, others)
        || !(tg.is_exists(duel, chk2, 1, others) || ntg.is_exists(duel, chk2, 1, others) || sg.is_exists(duel, chk2, 1, others))
    {
        return (false, none, none);
    }
    (true, tg.filter(duel, chk, None), ntg.filter(duel, chk, None))
}
