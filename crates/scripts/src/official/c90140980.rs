// Ojama King
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};
use engine::constants::MSG_FIELD_DISABLED;

const ID: u32 = 90140980;

pub const MATERIAL_SETCODE: &[u16] = &[SET_OJAMA];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(12482652),
        FusionMaterial::Code(42941100),
        FusionMaterial::Code(79335209),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization from Ojama Green, Yellow and Black in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (24094653, HAND), (12482652, HAND), (42941100, HAND), (79335209, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_FIELD_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_DISABLE_FIELD);
    e1.set_returning_operation(duel, disop);
    c.register_effect(duel, e1);
}

fn disop(duel: &mut Duel, a: &Args) -> Script<u32> {
    let tp = a.tp;
    let count = duel.get_location_count_full(1 - tp, LOCATION_MZONE, PLAYER_NONE, 0, 0xff);
    if count == 0 {
        return Ok(0);
    }
    let mut disabled = duel.select_disable_field(tp, 1, 0, LOCATION_MZONE as u16, 0)?.unwrap_or(0);
    if count > 1 && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        disabled |= duel.select_disable_field(tp, 1, 0, LOCATION_MZONE as u16, disabled)?.unwrap_or(0);
        if count > 2 && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            disabled |= duel.select_disable_field(tp, 1, 0, LOCATION_MZONE as u16, disabled)?.unwrap_or(0);
        }
    }
    Ok(disabled)
}
