// Destiny HERO - Dusktopia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 93657021;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_DESTINY_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), sumtype, tp)),
    ];
}

struct Params;

impl FusionSummonParams for Params {}

// In the Extra Deck, Fusion Summoned by Polymerization with Destroyer Phoenix Enforcer and Malicious, whose trigger Fusion
// Summons another Enforcer with Dreadmaster and a Malicious.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (24094653, HAND),
    (60461804, MZONE),
    (9411399, MZONE),
    (60461804, EXTRA),
    (40591390, MZONE),
    (9411399, HAND),
    (40591390, HAND),
    (40591390, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && !duel.is_phase(PHASE_DAMAGE)
    });
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, indtg);
    e2.set_operation(duel, indop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp) && c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), sumtype, tp)
}

fn indtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn indop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3008);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        tc.register_effect(duel, e2);
        let e3 = e1.clone_effect(duel);
        e3.set_description(duel, 3213);
        e3.set_code(duel, EFFECT_NO_BATTLE_DAMAGE);
        tc.register_effect(duel, e3);
        let e4 = e1.clone_effect(duel);
        e4.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
        tc.register_effect(duel, e4);
    }
    Ok(())
}
