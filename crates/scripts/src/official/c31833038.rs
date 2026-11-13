// Borreload Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 31833038;

// In the Extra Deck, Link Summoned from three Rokket effect monsters, attacking Sangans of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (25554552, MZONE),
    (25554552, MZONE),
    (25554552, MZONE),
    (68464358, MZONE),
    (68464358, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

struct SendToGrave;

impl aux::DelayedOperation for SendToGrave {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_grave(ag, REASON_EFFECT)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(turn_key(a.e.get_label(duel))) + 1
    }
}

// The turn the control changed in, per delayed effect (its first label is the fid).
fn turn_key(fid: i64) -> u64 {
    crate::keys::delayed_turn(ID, fid)
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP | TIMING_END_PHASE, TIMING_DAMAGE_STEP | TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, aux::stat_change_damage_step_condition);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_CONTROL);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_condition(duel, ctcon);
    e3.set_target(duel, cttg);
    e3.set_operation(duel, ctop);
    c.register_effect(duel, e3);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_chain_limit(|_, _, ep, tp, _| tp == ep, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, -500);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let zone = c.get_linked_zone(duel) & ZONES_MMZ;
    duel.get_attacker() == Some(c)
        && duel
            .get_attack_target()
            .is_some_and(|tc| tc.is_controler(duel, 1 - a.tp) && tc.is_controler_can_be_changed_full(duel, false, zone))
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tc = duel.get_attack_target();
    if let Some(tc) = tc {
        duel.set_target_card(tc);
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, tc.map(Into::into), 1, a.tp, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none() && c.is_relate_to_effect(duel, e));
    let Some(tc) = tc else { return Ok(()) };
    if !(c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && tc.is_controler(duel, 1 - tp)) {
        return Ok(());
    }
    let zone = c.get_linked_zone(duel) & ZONES_MMZ;
    // The core returns the control result as an integer, which is truthy in Lua even when 0.
    let reason_player = duel.get_reason_player();
    duel.get_control_full(tc, tp, 0, 0, zone, reason_player)?;
    let turn = i64::from(duel.get_turn_count());
    if let Some(delayed) = aux::delayed_operation_full::<SendToGrave>(duel, tc.into(), PHASE_END, RESET_PHASE | PHASE_END, ID, e, tp, 0, 2)
    {
        duel.set_script_state(turn_key(delayed.get_label(duel)), turn);
    }
    Ok(())
}
