// Number 1: Numeron Gate Ekam
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 15232745;
pub const XYZ_NUMBER: u32 = 1;

// In the Extra Deck, Xyz Summoned with Level 1 Normal Monsters, with the opponent's Elfnote Lucina and Ash Blossom to attack into it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (3557275, MZONE),
    (7902349, MZONE),
    (8124921, MZONE),
    (13597785, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_effects(duel, c, ID);
}

// The effects Number 4: Numeron Gate Catvari shares, which differ only in the description's code.
pub(super) fn add_effects(duel: &mut Duel, c: CardId, id: u32) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(id, 0));
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

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_OPPO_BATTLE) && c.is_relate_to_battle(duel)
}

fn numeron_filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_NUMERON])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(numeron_filter, a.tp, LOCATION_MZONE, 0, 1, None));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(numeron_filter, a.tp, LOCATION_MZONE, 0, None);
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
