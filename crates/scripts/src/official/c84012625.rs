// Cosmic Flare
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 84012625;

// With Stardust Dragon to tribute, against the opponent's Raigeki, and a second copy in the Graveyard to banish instead.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (44508094, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START);
    e1.set_condition(duel, |duel, _| {
        duel.is_able_to_enter_bp() || (duel.is_battle_phase() && !duel.is_phase(PHASE_BATTLE))
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_COST_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target_range(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_able_to_remove_as_cost(duel));
    e2.set_value(duel, Value::CostReplace(repval));
    e2.set_operation(duel, |duel, a| {
        duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_COST | REASON_REPLACE)?;
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_STARDUST]) && c.is_type(duel, TYPE_SYNCHRO) && c.is_faceup(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, tgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        tc.register_flag_effect_full(
            duel,
            ID,
            RESETS_STANDARD_PHASE_END,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_BATTLE_START);
        e1.set_condition(duel, tdcon);
        e1.set_operation(duel, tdop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    let (Some(bc1), _) = duel.get_battle_monster(a.tp) else {
        return false;
    };
    bc1.is_type(duel, TYPE_SYNCHRO)
        && bc1.is_faceup(duel)
        && bc1.has_flag_effect(duel, ID)
        && bc1.get_battle_target(duel).is_some_and(|bc2| bc2.is_controler(duel, 1 - a.tp))
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let (Some(bc1), _) = duel.get_battle_monster(a.tp)
        && let Some(bc2) = bc1.get_battle_target(duel)
    {
        duel.send_to_deck(bc2, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn repval(duel: &Duel, base: EffectId, call: CostReplaceCall) -> bool {
    let a = call.args;
    let c = a.e.get_handler(duel);
    c.is_set_card(duel, &[SET_STARDUST])
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_controler(duel, a.tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
        && (call.extracon)(duel, base, a)
}
