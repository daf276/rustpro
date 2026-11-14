// DoomZ Command "A.D.R.A.S.T.E.I.A."
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 84054556;

pub const LISTED_SERIES: &[u16] = &[SET_DOOMZ];

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand and Graveyard, to equip DoomZ monsters (Elara, Amalthe) on the field, and Elara and Drastea in the hand and
// Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (28877382, MZONE),
    (54265980, MZONE),
    (54265980, HAND),
    (92472273, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_count_limit(duel, 1, 0);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE > 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_EQUIP + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, eqtg);
    e3.set_operation(duel, eqop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ])
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ec = e.get_handler(duel).get_equip_target(duel);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, ec.map(Into::into)) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, ec.map(Into::into), 1, tp, 0);
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(ec) = e.get_handler(duel).get_equip_target(duel)
        && duel.destroy(ec, REASON_EFFECT)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            1,
            Some(ec.into()),
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}

fn level_or_rank(tc: CardId, duel: &Duel) -> i32 {
    if tc.has_level(duel) {
        tc.get_level(duel) as i32
    } else if tc.has_rank(duel) {
        tc.get_rank(duel) as i32
    } else {
        0
    }
}

// The Lua's chkc branch reads the undefined global `c`, so it errors and the check fails.
fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let lv_rnk = g.and_then(|g| g.get_first()).map_or(0, |tc| level_or_rank(tc, duel));
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, lv_rnk * 100);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && duel.equip(tp, c, tc)?
    {
        let lv_rnk = level_or_rank(tc, duel);
        duel.break_effect()?;
        duel.damage(tp, (lv_rnk * 100) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
