// Blackwing Armor Master
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 69031175;
const WEDGE_COUNTER: u16 = 0x1002;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_BLACKWING], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Copies on the field and in the Extra Deck (Synchro Summoned from Gale, a Level 3 Tuner, and Bora, Level 4) attacking the
// opponent's Blue-Eyes White Dragons, which survive the battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (2009101, MZONE),
    (49003716, HAND),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_COUNTER);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_DAMAGE_STEP_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, condition1);
    e3.set_operation(duel, operation1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, cost2);
    e4.set_operation(duel, operation2);
    c.register_effect(duel, e4);
}

fn condition1(duel: &Duel, a: &Args) -> bool {
    let atg = duel.get_attack_target();
    duel.get_attacker() == Some(a.e.get_handler(duel))
        && atg.is_some_and(|atg| atg.is_relate_to_battle(duel) && atg.get_counter(duel, WEDGE_COUNTER) == 0)
}

fn operation1(duel: &mut Duel, _: &Args) -> Script<()> {
    if let Some(atg) = duel.get_attack_target().filter(|atg| atg.is_relate_to_battle(duel)) {
        atg.add_counter(duel, WEDGE_COUNTER, 1);
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_counter(duel, WEDGE_COUNTER) > 0
}

fn cost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(filter, tp, 0, LOCATION_MZONE, None);
    for t in g {
        let count = t.get_counter(duel, WEDGE_COUNTER);
        t.remove_counter(duel, tp, WEDGE_COUNTER, count, REASON_COST)?;
    }
    duel.set_target_card(g);
    Ok(false)
}

fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    for t in g.unwrap_or_default() {
        if t.is_relate_to_effect(duel, a.e) {
            for code in [EFFECT_SET_ATTACK_FINAL, EFFECT_SET_DEFENSE_FINAL] {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, code);
                e1.set_value(duel, 0);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                t.register_effect(duel, e1);
            }
        }
    }
    Ok(())
}
