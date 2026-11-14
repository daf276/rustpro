// Spright Pixies
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 49928686;

// Psychic Kappas (Level 2) to summon it and to battle Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (7892180, MZONE),
    (7892180, MZONE),
    (7892180, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e2.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, atkcon);
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_level(duel, &[2]) || c.is_rank(duel, &[2]))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let (Some(at), Some(d)) = duel.get_battle_monster(a.tp) else {
        return false;
    };
    at.is_faceup(duel)
        && d.is_faceup(duel)
        && at != a.e.get_handler(duel)
        && (at.is_level(duel, &[2]) || at.is_rank(duel, &[2]) || at.is_link(duel, &[2]))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (Some(at), Some(d)) = duel.get_battle_monster(a.tp) else {
        return Ok(());
    };
    if at.is_relate_to_battle(duel)
        && at.is_faceup(duel)
        && d.is_relate_to_battle(duel)
        && d.is_faceup(duel)
        && d.is_controler(duel, 1 - a.tp)
    {
        let atk = d.get_attack(duel);
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        at.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        at.register_effect(duel, e2);
    }
    Ok(())
}
