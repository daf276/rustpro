// Gladiator Beast Domitianus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 33652635;
const VESPASIUS: u32 = 88996322;

pub const MATERIAL_SETCODE: &[u16] = &[SET_GLADIATOR];
pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(VESPASIUS),
        FusionMaterial::Filter(gladiator),
        FusionMaterial::Filter(gladiator),
    ];
}

fn gladiator(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_GLADIATOR], Some(fc), sumtype, tp)
}

// Contact Summoned from Vespasius and Gladiator Beast Hoplomus on the field, with Hoplomus in the Deck, against weak monsters
// and an attacker of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (VESPASIUS, MZONE),
    (4253484, MZONE),
    (4253484, DECK),
    (4253484, DECK),
    (13597785, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<super::c27346636::Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, discon);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_PATRICIAN_OF_DARKNESS);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 0, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e3.set_cost(duel, super::c62000467::spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| filter(c, duel, a.e, a.tp), 100, false, 0)
    });
    c.register_effect(duel, e3);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_negatable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_relate_to_effect(duel, re) && rc.is_destructable(duel) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 100, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > -1
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}
