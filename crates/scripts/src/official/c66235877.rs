// Fiend Skull Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 66235877;
pub const MATERIAL_SETCODE: &[u16] = &[SET_ARCHFIEND];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(93220472), FusionMaterial::Code(16475472)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Cave Dragon and Lesser Fiend.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (CARD_POLYMERIZATION, HAND), (93220472, HAND), (16475472, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_FLIP));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, ID);
    e3.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else {
        return Ok(());
    };
    let ev = a.ev as u8;
    if re.is_active_type(duel, TYPE_FLIP) {
        duel.negate_effect(ev);
    }
    if re.is_trap_effect(duel) && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        let g = duel.get_chain_info::<Option<CardSet>>(ev, CHAININFO_TARGET_CARDS);
        if g.is_some_and(|g| g.is_contains(a.e.get_handler(duel))) {
            duel.negate_effect(ev);
        }
    }
    Ok(())
}
