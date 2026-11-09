// Spirit Ryu
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 67957315;

// Two on the field with Luster Dragon, a Dragon Normal Monster, in the hand to discard, and Sangans of the opponent's to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (11091375, HAND),
    (11091375, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_BATTLE_PHASE | TIMING_CHAIN_END, TIMING_BATTLE_PHASE | TIMING_CHAIN_END);
    e1.set_condition(duel, adcon);
    e1.set_cost(duel, adcost);
    e1.set_operation(duel, adop);
    c.register_effect(duel, e1);
}

fn adcon(duel: &Duel, a: &Args) -> bool {
    let c = Some(a.e.get_handler(duel));
    duel.is_turn_player(a.tp) && duel.is_battle_phase() && (c == duel.get_attacker() || c == duel.get_attack_target())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_discardable(duel) && c.is_able_to_grave_as_cost(duel)
}

fn adcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn adop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_BATTLE);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        c.register_effect(duel, e2);
    }
    Ok(())
}
