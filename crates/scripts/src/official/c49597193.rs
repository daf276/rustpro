// Gem-Knight Topaz
use super::c3113836::{gem_knight, register_splimit};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 49597193;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(54620698), FusionMaterial::Filter(gem_knight)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Gem-Knight Tourmaline and Garnet; on the field against the opponent's
// weak Gem-Knight Lapis.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (54620698, HAND),
    (91731841, HAND),
    (99645428, MZONE | OPPONENT),
    (99645428, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    register_splimit(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_condition(duel, aux::bdgcon);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, stock::damage);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_EXTRA_ATTACK);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    let dam = bc.map_or(0, |bc| bc.get_attack(duel)).max(0);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}
