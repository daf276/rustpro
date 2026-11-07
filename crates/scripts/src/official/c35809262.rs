// Elemental HERO Flame Wingman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 35809262;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(21844576), FusionMaterial::Code(58932615)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Avian and Burstinatrix (Normal Monsters), and on
// the field to battle weak Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (21844576, HAND),
    (58932615, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_battle_target(duel)
        .is_some_and(|bc| c.is_relate_to_battle(duel) && bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    duel.set_target_card(bc);
    let dam = bc.get_attack(duel).max(0);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        duel.damage(p, tc.get_attack(duel).max(0) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
