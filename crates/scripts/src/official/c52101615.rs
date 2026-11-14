// The Dark - Hex-Sealed Fusion
use engine::OperationFn;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 52101615;
const ALBION: u32 = 87746184;

// Albion the Branded Dragon in the Extra Deck, with Petit Angel as the LIGHT material beside this card standing in for Fallen of Albaz.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ALBION, EXTRA), (38142739, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ALBION)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID, ATTRIBUTE_DARK>(duel, c, |duel, a| operation(duel, a, a.tp));
}

// The effects The Dark and The Light - Hex-Sealed Fusion share; the Light's operation checks the zones from the activating player.
pub fn register<const CODE: u32, const ATTRIBUTE: u32>(duel: &mut Duel, c: CardId, operation: OperationFn) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_description(duel, aux::stringid(CODE, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_label(duel, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target::<ATTRIBUTE>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_FUSION_SUBSTITUTE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .is_location(duel, LOCATION_GRAVE | LOCATION_SZONE | LOCATION_MZONE | LOCATION_HAND)
    });
    c.register_effect(duel, e2);
}

fn filter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8, m: CardSet, gc: CardId, chkf: u64) -> bool {
    c.is_type(duel, TYPE_FUSION)
        && c.is_attribute(duel, ATTRIBUTE)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.check_fusion_material(duel, Some(m), Some(gc.into()), chkf)
}

fn cost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 1);
    Ok(true)
}

fn fil(c: CardId, duel: &Duel) -> bool {
    c.is_can_be_fusion_material(duel, None, MATERIAL_FUSION)
        && c.is_has_effect(duel, EFFECT_EXTRA_RELEASE_NONSUM).is_some()
        && c.is_releasable(duel)
}

fn fil2(c: CardId, duel: &Duel) -> bool {
    c.is_can_be_fusion_material(duel, None, MATERIAL_FUSION) && c.is_releasable(duel)
}

// The materials of the field and the opponent's monsters that can be released for EFFECT_EXTRA_RELEASE_NONSUM. Those and the
// check that at most one of them is used (Fusion.CheckAdditional) have no card yet.
fn materials(duel: &Duel, tp: u8) -> CardSet {
    let mg = duel.get_matching_group(fil2, tp, LOCATION_MZONE, 0, None);
    let mg2 = duel.get_matching_group(fil, tp, 0, LOCATION_MZONE, None);
    unported!(duel, !mg2.is_empty());
    mg.union(mg2)
}

fn target<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let chkf = u64::from(tp) | FUSPROC_NOTFUSION;
    if chk == 0 {
        if e.get_label(duel) != 1 || !c.is_releasable(duel) {
            return Ok(false);
        }
        e.set_label(duel, 0);
        let m = materials(duel, tp);
        return Ok(duel.is_existing_matching_card(
            |sc, duel| filter::<ATTRIBUTE>(sc, duel, e, tp, m, c, chkf),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    let m = materials(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| filter::<ATTRIBUTE>(sc, duel, e, tp, m, c, chkf),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else {
        return Ok(false);
    };
    let mat = duel.select_fusion_material(tp, tc, m, Some(c.into()), chkf)?;
    duel.release(mat, REASON_COST)?;
    e.set_label(duel, i64::from(tc.get_code(duel)));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

pub fn operation(duel: &mut Duel, a: &Args, rp: u8) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let code = e.get_label(duel) as u32;
    let filter2 = |c: CardId, duel: &Duel| {
        c.is_code(duel, code)
            && duel.get_location_count_from_ex(tp, rp, None, c) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    };
    if let Some(tc) = duel.get_first_matching_card(filter2, tp, LOCATION_EXTRA, 0, None) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
