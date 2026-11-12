// Combat Wheel
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 12501230;
const COUNTER_COMBAT_WHEEL: u16 = 0x20e;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with a Mystical Elf to gain ATK from and cards in hand to discard, so that the opponent's Blue-Eyes White Dragons
// can battle it down below their ATK once it has a counter.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (15025844, MZONE),
    (26202165, HAND),
    (26202165, HAND),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_COMBAT_WHEEL);
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_value(
        duel,
        Value::Reason(|duel, e, _, r, rp| r & REASON_EFFECT == REASON_EFFECT && rp == 1 - e.get_handler_player(duel)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_COUNTER);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    let timing = TIMING_BATTLE_START | TIMING_BATTLE_END | TIMING_DAMAGE_STEP;
    e2.set_hint_timing(duel, timing, timing);
    e2.set_condition(duel, atkcon);
    e2.set_cost(duel, atkcost);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e4a = duel.create_effect(c);
    e4a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4a.set_code(duel, EVENT_LEAVE_FIELD_P);
    e4a.set_operation(duel, |duel, a| {
        let counters = i64::from(a.e.get_handler(duel).get_counter(duel, COUNTER_COMBAT_WHEEL));
        a.e.set_label(duel, counters);
        Ok(())
    });
    c.register_effect(duel, e4a);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_BATTLE_DESTROYED);
    e4.set_label_object(duel, Some(e4a));
    e4.set_condition(duel, |duel, a| {
        a.e.get_label_object(duel).is_some_and(|e4a| e4a.get_label(duel) > 0)
    });
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && duel.is_battle_phase() && aux::stat_change_damage_step_condition(duel, a)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID)
            && c.is_can_add_counter(duel, COUNTER_COMBAT_WHEEL, 1)
            && duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE, 0, 1);
    Ok(false)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, te, tc| tc != te.get_handler(duel));
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    c.register_effect(duel, e1);
    let atk = duel
        .get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, Some(c.into()))
        .get_sum(duel, |tc, duel| i64::from(tc.get_attack(duel)));
    if atk > 0 && c.update_attack(duel, (atk / 2) as i32, None, None, None) > 0 && c.is_can_add_counter(duel, COUNTER_COMBAT_WHEEL, 0) {
        duel.break_effect()?;
        c.add_counter(duel, COUNTER_COMBAT_WHEEL, 1);
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, 0, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
