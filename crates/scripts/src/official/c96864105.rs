// Number C73: Abyss Supra Splash
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 96864105;
pub const XYZ_NUMBER: u32 = 73;
const ABYSS_SPLASH: u32 = 36076683;

pub const LISTED_NAMES: &[u32] = &[ABYSS_SPLASH];

// In the Extra Deck, Xyz Summoned with three Level 6 Normal Monsters (Frostosaurus), with the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, Cost::soft_once_per_battle::<ID>])
    });
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_code(duel, ABYSS_SPLASH), 1, None)
    });
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let (bc1, bc2) = duel.get_battle_monster(a.tp);
    bc1.is_some() && bc2.is_some_and(|bc2| bc2.get_attack(duel) > 0)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (bc1, bc2) = duel.get_battle_monster(a.tp);
    if let (Some(bc1), Some(bc2)) = (bc1, bc2)
        && bc1.is_relate_to_battle(duel)
        && bc1.is_faceup(duel)
        && bc2.is_relate_to_battle(duel)
        && bc2.is_faceup(duel)
    {
        let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE_CAL;
        let atk = bc2.get_attack(duel);
        bc1.update_attack(duel, atk, Some(reset), Some(a.e.get_handler(duel)), None);
    }
    Ok(())
}
