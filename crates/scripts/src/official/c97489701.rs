// Red Nova Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 97489701;
pub const MATERIAL: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];
pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN1: usize = 2;
    const MAX1: usize = 2;
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND)
    }
}

// In the Extra Deck, Synchro Summoned with Red Dragon Archfiend and two Level 2 Tuners on the field, and on the field to
// banish itself against the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (17266660, MZONE),
    (17266660, MZONE),
    (92919429, MZONE),
    (92919429, MZONE),
    (13597785, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(|duel, _, c| atkval(duel, c)));
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
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, nacon);
    e3.set_target(duel, natg);
    e3.set_operation(duel, naop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_range(duel, LOCATION_REMOVED);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e5.set_code(duel, EFFECT_MULTIPLE_TUNERS);
    c.register_effect(duel, e5);
}

// Shared with Red Hypernova Dragon.
pub fn atkval(duel: &Duel, c: CardId) -> i32 {
    500 * duel.get_matching_group_count(
        |c, duel| c.is_type(duel, TYPE_TUNER),
        c.get_controler(duel),
        LOCATION_GRAVE,
        0,
        None,
    ) as i32
}

fn nacon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker().is_some_and(|c| c.is_controler(duel, 1 - a.tp))
}

fn natg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let attacker = duel.get_attacker();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == attacker);
    }
    let c = a.e.get_handler(duel);
    let Some(attacker) = attacker else { return Ok(false) };
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && attacker.is_can_be_effect_target(duel, a.e) && !c.is_status(duel, STATUS_CHAINING));
    }
    duel.set_target_card(attacker);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn naop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_attack()?;
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.remove(c, POS_FACEUP, REASON_EFFECT)? != 0 {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
