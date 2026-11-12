// Superheavy Samurai Stealth Ninja
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 50065971;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_MACHINE, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_MACHINE, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned from Motorbike (a Level 2 Tuner) and Shapesnatch (Level 5), with Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (83334932, MZONE),
    (83334932, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, dircon);
    e2.set_operation(duel, dirop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_operation(duel, spreg);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, spcon);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    e4.set_label_object(duel, Some(e3));
    c.register_effect(duel, e4);
}

fn dircon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(CardId::is_spell_trap, a.tp, LOCATION_GRAVE, 0, 1, None)
        && a.e.get_handler(duel).is_has_effect(duel, EFFECT_DIRECT_ATTACK).is_none()
}

fn dirop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_DEFENSE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, (c.get_base_defense(duel) + 1) / 2);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DIRECT_ATTACK);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn spreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if a.r & (REASON_EFFECT + REASON_DESTROY) != REASON_EFFECT + REASON_DESTROY {
        return Ok(());
    }
    if duel.is_phase(PHASE_STANDBY) {
        let turn = i64::from(duel.get_turn_count());
        e.set_label(duel, turn);
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 0, 2);
    } else {
        e.set_label(duel, 0);
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 0, 1);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel)
        .is_some_and(|e3| e3.get_label(duel) != i64::from(duel.get_turn_count()))
        && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}
