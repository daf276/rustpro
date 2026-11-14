// Despian Proskenion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 18666161;

pub const LISTED_SERIES: &[u16] = &[SET_DESPIA];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DESPIA], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp)),
    ];
}

const BLOOM_HARMONIST: u32 = 34974462;

// In the Extra Deck, Fusion Summoned by Polymerization from two Aluber (Despia, DARK) and Aria (LIGHT), with Bloom Harmonist
// (a Link Monster) in the opponent's GY and Sangan to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (62962630, MZONE),
    (62962630, MZONE),
    (40502912, MZONE),
    (BLOOM_HARMONIST, GRAVE | OPPONENT),
    (BLOOM_HARMONIST, GRAVE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(BLOOM_HARMONIST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    // The Lua's SetCode(CATEGORY_REMOVE+CATEGORY_SPECIAL_SUMMON) is overwritten by the next SetCode, so the effect has no category.
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::bdocon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lc: i32) -> bool {
    c.is_type(duel, TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ + TYPE_LINK)
        && (c.is_able_to_remove(duel) || (lc > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lc = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && tgfilter(chkc, duel, e, tp, lc));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, e, tp, lc), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c, duel| tgfilter(c, duel, e, tp, lc), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 1 - tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, g, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let b1 = tc.is_able_to_remove(duel);
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 0)), (b2, aux::stringid(ID, 1))])? {
        Some(1) => {
            duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
        }
        Some(2) => {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        _ => {}
    }
    Ok(())
}

fn damage(bc: CardId, duel: &Duel) -> i32 {
    bc.get_text_attack(duel).max(bc.get_text_defense(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    let dam = damage(bc, duel);
    if chk == 0 {
        return Ok(dam > 0);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        let dam = damage(bc, duel).max(0);
        duel.damage(p, dam as u32, REASON_EFFECT)?;
    }
    Ok(())
}
