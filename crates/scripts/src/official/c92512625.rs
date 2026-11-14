// Solemn Scolding
use crate::prelude::*;
use crate::testing::SZONE;

const ID: u32 = 92512625;

// Set, as the only face-down card in its Spell & Trap Zone.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_SUMMON);
    e1.set_condition(duel, condition1);
    e1.set_cost(duel, Cost::pay_lp::<3000>);
    e1.set_target(duel, super::c41420027::target1);
    e1.set_operation(duel, super::c41420027::activate1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_condition(duel, condition2);
    e4.set_cost(duel, Cost::pay_lp::<3000>);
    e4.set_target(duel, stock::negate_destroy_target);
    e4.set_operation(duel, super::c41420027::activate2);
    c.register_effect(duel, e4);
}

fn no_other_set_card(duel: &Duel, a: &Args) -> bool {
    let cfilter = |c: CardId, duel: &Duel| c.is_facedown(duel) && c.get_sequence(duel) < 5;
    !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_SZONE, 0, 1, Some(a.e.get_handler(duel).into()))
}

fn condition1(duel: &Duel, a: &Args) -> bool {
    duel.get_current_chain_full(true) == 0 && no_other_set_card(duel, a)
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && no_other_set_card(duel, a)
}
