// Red Supernova Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 99585850;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN1: usize = 3;
    const MAX1: usize = 3;
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_type(duel, TYPE_SYNCHRO))
    }
}

// In the Extra Deck, Synchro Summoned with three Level 2 Tuners (Guardragon Justicia) and a Level 6 Synchro Monster (Goyo Guardian),
// with Elfnote Lucina on the opponent's side to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (11012154, MZONE),
    (11012154, MZONE),
    (11012154, MZONE),
    (7391448, MZONE),
    (7391448, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e3.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
    });
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_PHASE + PHASE_END);
    e5.set_range(duel, LOCATION_REMOVED);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, spcon);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
    e3.set_label_object(duel, Some(e5));
    e4.set_label_object(duel, Some(e5));
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e6.set_code(duel, EFFECT_MULTIPLE_TUNERS);
    c.register_effect(duel, e6);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let tuner = |c: CardId, duel: &Duel| c.is_type(duel, TYPE_TUNER);
    duel.get_matching_group_count(tuner, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 500
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    if c.is_relate_to_effect(duel, e) && c.is_able_to_remove(duel) {
        g.insert(c);
    }
    if !g.is_empty() && duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0 {
        let og = duel.get_operated_group();
        if og.is_contains(c) {
            let mut ct = 1;
            let turn = duel.get_turn_count();
            let respawn = e.get_label_object(duel);
            if duel.is_turn_player(tp) && duel.is_phase(PHASE_END) {
                ct = 2;
                respawn.inspect(|e5| e5.set_label(duel, i64::from(turn)));
            } else {
                respawn.inspect(|e5| e5.set_label(duel, 0));
            }
            c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END | RESET_SELF_TURN, 0, ct);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, ID) != 0
        && a.e.get_label(duel) != i64::from(duel.get_turn_count())
        && duel.is_turn_player(a.tp)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
