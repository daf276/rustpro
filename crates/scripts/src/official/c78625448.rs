// Number 3: Numeron Gate Trini
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 78625448;

pub const LISTED_SERIES: &[u16] = &[SET_NUMERON];
pub const XYZ_NUMBER: u32 = 3;

// In the Extra Deck, Xyz Summoned with three Level 1 Normal Monsters (Petit Moth), or on the field with Sangan attached by Xyz Import,
// with Cyber Falcon to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (58192742, MZONE),
    (58192742, MZONE),
    (58192742, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn numeron(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_NUMERON])
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_OPPO_BATTLE) && c.is_relate_to_battle(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(numeron, a.tp, LOCATION_MZONE, 0, 1, None));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(numeron, a.tp, LOCATION_MZONE, 0, None);
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, tc.get_attack(duel) * 2);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 1);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
