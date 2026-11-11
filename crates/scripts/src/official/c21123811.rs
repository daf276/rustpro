// Cosmic Blazar Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 21123811;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

// On the field and in the Extra Deck, Synchro Summoned from Formula Synchron (Level 2) and two Level 5 Synchro Monsters (Magical
// Android, Ally of Justice Catastor), against the opponent's Raigeki and Summoned Skull.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (50091196, MZONE),
    (43385557, MZONE),
    (26593852, MZONE),
    (12580477, HAND | OPPONENT),
    (70781052, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_count_limit(duel, 1, 0);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, cost);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_SUMMON);
    e3.set_condition(duel, |duel, a| a.tp != a.ep && duel.get_current_chain() == 0);
    e3.set_cost(duel, cost);
    e3.set_target(duel, dstg);
    e3.set_operation(duel, dsop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_FLIP_SUMMON);
    c.register_effect(duel, e4);
    let e5 = e3.clone_effect(duel);
    e5.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 2));
    e6.set_type(duel, EFFECT_TYPE_QUICK_O);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e6.set_count_limit(duel, 1, 0);
    e6.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e6.set_cost(duel, cost);
    e6.set_operation(duel, negop);
    c.register_effect(duel, e6);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp != a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel));
    }
    if duel.remove(c, POS_FACEUP, REASON_COST + REASON_TEMPORARY)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_label_object_group(duel, Some(c.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn dstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn dsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_attack()? {
        duel.skip_phase(1 - a.tp, PHASE_BATTLE, RESET_PHASE | PHASE_BATTLE, 1);
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(c) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(c)?;
    }
    Ok(())
}
