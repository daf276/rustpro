// Bloom Diva the Melodious Choir
use super::c56208713::melodious;
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE};

const ID: u32 = 84988419;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS_MAESTRA, SET_MELODIOUS];
pub const MATERIAL_SETCODE: &[u16] = &[SET_MELODIOUS, SET_MELODIOUS_MAESTRA];

pub fn maestra(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_MELODIOUS_MAESTRA], Some(fc), sumtype, tp)
}

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(maestra), FusionMaterial::Filter(melodious)];
}

// Copies on the field for the opponent's Special Summoned Elfnote monsters to attack, and in the Extra Deck for Polymerization
// with Bacha and Aria.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (56208713, MZONE),
    (40502912, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    for code in [EFFECT_INDESTRUCTABLE_EFFECT, EFFECT_AVOID_BATTLE_DAMAGE] {
        let e = e1.clone_effect(duel);
        e.set_code(duel, code);
        c.register_effect(duel, e);
    }
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BATTLED);
    e4.set_condition(duel, condition);
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_battle_target(duel).is_some_and(|bc| {
        bc.get_summon_type(duel) & SUMMON_TYPE_SPECIAL == SUMMON_TYPE_SPECIAL && c.get_base_attack(duel) != bc.get_base_attack(duel)
    })
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(false) };
    if chk == 0 {
        return Ok(bc.is_relate_to_battle(duel));
    }
    let atk = (c.get_base_attack(duel) - bc.get_base_attack(duel)).abs();
    duel.set_target_card(bc);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, atk);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(bc.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(bc) = duel.get_first_target() else { return Ok(()) };
    let atk = (a.e.get_handler(duel).get_base_attack(duel) - bc.get_base_attack(duel)).abs();
    if bc.is_relate_to_effect(duel, a.e) && bc.is_faceup(duel) && duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)? != 0 {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
