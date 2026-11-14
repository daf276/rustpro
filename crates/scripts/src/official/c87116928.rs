// Chimeratech Megafleet Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 87116928;

pub const LISTED_SERIES: &[u16] = &[SET_CYBER_DRAGON];
pub const MATERIAL_SETCODE: &[u16] = &[SET_CYBER, SET_CYBER_DRAGON];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::Filter(|c, duel, _, _, tp| {
        c.get_sequence(duel) > 4 && c.is_location(duel, LOCATION_MZONE) && (c.is_controler(duel, tp) || c.is_faceup(duel))
    });
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
        c.is_set_card_full(duel, &[SET_CYBER_DRAGON], Some(fc), sumtype, tp)
    })];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(|c, duel| cfilter(c, duel, tp), tp, LOCATION_MZONE, LOCATION_MZONE, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, c: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
        e1.set_value(duel, g.len() as i32 * 1200);
        c.register_effect(duel, e1);
        Ok(())
    }
}

// In the Extra Deck, Special Summoned by contact from Cyber Twin Dragon (Fusion Summoned by Polymerization into the Extra Monster Zone) and Cyber Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (74157028, EXTRA),
    (74157028, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    e.get_handler(duel).get_location(duel) != LOCATION_EXTRA
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel) && (c.is_controler(duel, tp) || c.is_faceup(duel))
}
