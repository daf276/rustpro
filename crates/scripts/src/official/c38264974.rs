// Chimera the Illusion Beast
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 38264974;
pub const LISTED_NAMES: &[u32] = &[CARD_CHIMERA_MYTHICAL_BEAST];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_ILLUSION, Some(fc), sumtype, tp));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_CHIMERA_MYTHICAL_BEAST)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Chimera the Flying Mythical Beast and Illusion monsters (Jongleur-Ghoul
// Illusionist), with the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_CHIMERA_MYTHICAL_BEAST, MZONE),
    (47894537, HAND),
    (47894537, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_CHIMERA_MYTHICAL_BEAST as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, e, c| {
        let handler = e.get_handler(duel);
        c == handler || Some(c) == handler.get_battle_target(duel)
    });
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DAMAGE_STEP_END);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, atkcon);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let count = c.get_material(duel).len() as i32;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, count - 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE & !RESET_TOFIELD);
    c.register_effect(duel, e1);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return false };
    c.is_status(duel, STATUS_OPPO_BATTLE)
        && c.is_relate_to_battle(duel)
        && bc.is_relate_to_battle(duel)
        && (!bc.is_attack(duel, &[0]) || !bc.is_disabled(duel))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(()) };
    if bc.is_relate_to_battle(duel) && bc.is_faceup(duel) && bc.is_controler(duel, 1 - a.tp) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        bc.register_effect(duel, e1);
        bc.negate_effects(duel, c, 0, false);
    }
    Ok(())
}
