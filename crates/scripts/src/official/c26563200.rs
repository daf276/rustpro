// Muzurhythm the String Djinn
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 26563200;

pub const LISTED_SERIES: &[u16] = &[SET_DJINN];
// In the Extra Deck, Xyz Summoned with two Level 3 monsters (Necrolancer the Time-lord, Rock Ogre Grotto #1), and an
// opponent's monster (Sangan) to attack into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (61454890, MZONE),
    (61454890, MZONE),
    (68846917, MZONE),
    (68846917, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    if duel.get_current_phase() != PHASE_DAMAGE || duel.is_damage_calculated() {
        return false;
    }
    let tc = duel.get_attacker();
    unported!(duel, tc.is_none());
    tc.is_some_and(|tc| {
        tc.is_controler(duel, a.tp)
            && tc.is_relate_to_battle(duel)
            && tc.is_set_card(duel, &[SET_DJINN])
            && tc.is_type(duel, TYPE_XYZ)
            && duel.get_attack_target().is_some()
    })
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_attacker();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_battle(duel)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, tc.get_attack(duel) * 2);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
