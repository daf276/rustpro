// Dinomorphia Stealthbergia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 74936480;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(super::c48832775::ffilter); 2];
}

// Fusion Summoned by Polymerization with Therizia and Diplos; on the field against the opponent's Sangan and Raigeki, with
// Diplos in the GY to summon, and Domain to activate for free at 2000 LP or less.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (92133240, HAND),
    (38628859, HAND),
    (38628859, GRAVE),
    (38628859, GRAVE),
    (26631975, HAND),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_LPCOST_CHANGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.get_lp(a.e.get_handler_player(duel)) <= 2000);
    e1.set_value(duel, Value::LpCost(lpcostval));
    c.register_effect(duel, e1);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, damcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
    super::c92133240::register_destroyed_summon::<ID, 4, false>(duel, c);
}

fn lpcostval(duel: &Duel, _: EffectId, re: Option<EffectId>, _: u8, val: i32) -> i32 {
    let Some(re) = re else { return val };
    let rc = re.get_handler(duel);
    if re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && rc.is_trap(duel) {
        return 0;
    }
    if re.is_monster_effect(duel) && rc.is_set_card(duel, &[SET_DINOMORPHIA]) {
        return 0;
    }
    val
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re
            .is_some_and(|re| re.is_monster_effect(duel) && re.get_handler(duel).get_base_attack(duel) > 0)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let atk = a.re.map_or(0, |re| re.get_handler(duel).get_base_attack(duel));
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(atk as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, atk);
    Ok(false)
}
