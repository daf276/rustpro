// The Eye of Timaeus
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 1784686;

// Dark Magician, and Fusion Monsters that list it (Red-Eyes Dark Dragoon, Dark Cavalry, Dark Magician of Destruction).
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, MZONE),
    (37818794, EXTRA),
    (73452089, EXTRA),
    (59400890, EXTRA),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_ADD_CODE);
    e0.set_value(duel, 10000050);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARK_MAGICIAN])
        && c.is_can_be_fusion_material(duel, None, 0)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

pub fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    if duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) <= 0 {
        return false;
    }
    let mustg = aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION);
    c.is_type(duel, TYPE_FUSION)
        && c.lists_code_as_material(duel, &[mc.get_code(duel)])
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
        && (mustg.is_empty() || (mustg.len() == 1 && mustg.contains(mc)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FMATERIAL);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_can_be_fusion_material(duel, None, 0)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            sc.set_material(duel, Some(tc.into()));
            duel.send_to_grave(tc, REASON_EFFECT | REASON_MATERIAL | REASON_FUSION)?;
            duel.break_effect()?;
            // The Lua tests the number SpecialSummon returns, which is always truthy.
            duel.special_summon(sc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)?;
            sc.complete_procedure(duel);
        }
    }
    Ok(())
}
