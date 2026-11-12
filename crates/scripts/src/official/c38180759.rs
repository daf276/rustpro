// Unformed Void
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 38180759;

// In the Extra Deck, Xyz Summoned with three Level 4 Normal Monsters (Prevent Rat), with the opponent's Divine Dragon Knight Felgrand to add up.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (1639384, MZONE | OPPONENT),
    (1639384, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && duel.is_main_phase()
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(filter, a.tp, 0, LOCATION_MZONE, 1, None))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let g = duel.get_matching_group(filter, a.tp, 0, LOCATION_MZONE, None);
        let atk = g.get_sum(duel, |c, duel| c.get_attack(duel).into());
        if atk > 0 {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, atk as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            c.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
            c.register_effect(duel, e2);
        }
    }
    Ok(())
}
