// Number 107: Galaxy-Eyes Tachyon Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 88177324;

// In the Extra Deck, Xyz Summoned with Black Luster Soldier and Steam Knight - Hamburk, and the opponent's monsters with effects and Spell/Trap Cards to chain.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (23923758, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
// The opponent chaining in the same Battle Phase is too rare to assert.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Report(&Check::CardHint(3201))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_BATTLE_START);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_condition(duel, |duel, a| a.rp != a.tp && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel)
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.get_attack(duel) != c.get_base_attack(duel) || c.get_defense(duel) != c.get_base_defense(duel))
}

fn filter3(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = Some(a.e.get_handler(duel).into());
        return Ok(duel.is_existing_matching_card(filter1, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, c)
            || duel.is_existing_matching_card(filter2, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, c));
    }
    Ok(false)
}

fn register_reset_effect(duel: &mut Duel, c: CardId, tc: CardId, code: u32, value: Option<i32>) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, code);
    if let Some(value) = value {
        e1.set_value(duel, value);
    }
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(filter3, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    for tc in g {
        register_reset_effect(duel, c, tc, EFFECT_DISABLE, None);
        register_reset_effect(duel, c, tc, EFFECT_DISABLE_EFFECT, None);
    }
    let g = duel.get_matching_group(filter2, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    for tc in g {
        if tc.get_attack(duel) != tc.get_base_attack(duel) {
            register_reset_effect(duel, c, tc, EFFECT_SET_ATTACK_FINAL, Some(tc.get_base_attack(duel)));
        }
        if tc.get_defense(duel) != tc.get_base_defense(duel) {
            register_reset_effect(duel, c, tc, EFFECT_SET_DEFENSE_FINAL, Some(tc.get_base_defense(duel)));
        }
    }
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE, 0, 1);
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 1000);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_BATTLE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, 3201);
    e2.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_value(duel, 1);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
    c.register_effect(duel, e2);
    Ok(())
}
