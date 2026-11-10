// Chimeratech Fortress Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 79229522;
pub const MATERIAL_SETCODE: &[u16] = &[SET_CYBER, SET_CYBER_DRAGON];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::ContactFilter(fil);
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_CYBER_DRAGON)];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(|c, duel| cfilter(c, duel, tp), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, c: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
        e1.set_value(duel, g.len() as i32 * 1000);
        c.register_effect(duel, e1);
        Ok(())
    }
}

// Two Extra Deck copies, Cyber Dragons on the field and in the hand for their Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, 0),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn fil(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, contact: bool) -> bool {
    let sumtype = if contact { 0 } else { sumtype };
    c.is_race_full(duel, RACE_MACHINE, Some(fc), sumtype, tp) && (!contact || c.is_type_full(duel, TYPE_MONSTER, Some(fc), sumtype, tp))
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel) && (c.is_controler(duel, tp) || c.is_faceup(duel))
}
