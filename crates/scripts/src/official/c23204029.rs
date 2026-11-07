// Contrast HERO Chaos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 23204029;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_MASKED_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(masked_hero); 2];
}

fn masked_hero(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_MASKED_HERO], Some(fc), sumtype, tp)
}

// In the Extra Deck, Fusion Summoned by Polymerization with two Masked HERO monsters, and one on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, MZONE), (24094653, HAND), (10808715, MZONE), (58288218, HAND)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_ADD_ATTRIBUTE);
    e2.set_value(duel, ATTRIBUTE_LIGHT as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if ((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER)) && tc.is_relate_to_effect(duel, e) {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        super::c97268402::negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}
