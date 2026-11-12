// Lavalval Ignis
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 3989465;

// Copies in the Extra Deck and on the field, Xyz Summoned from Level 3 Normal Monsters (Dokuroyaiba, Flame Manipulator), and Sangan on
// the opponent's side to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (30325729, MZONE),
    (30325729, MZONE),
    (34460851, MZONE),
    (34460851, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, Cost::soft_once_per_battle::<ID>])
    });
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_relate_to_battle(duel) && duel.is_phase(PHASE_DAMAGE) && aux::stat_change_damage_step_condition(duel, a)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        c.update_attack(duel, 500, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
    }
    Ok(())
}
