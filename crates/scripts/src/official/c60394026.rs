// Ryzeal Plugin
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND};

const ID: u32 = 60394026;

pub const LISTED_SERIES: &[u16] = &[SET_RYZEAL];
// Copies in hand, Ryzeal Detonator and Ext Ryzeal in the Graveyard to summon, and Ryzeal monsters in the Deck to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (34909328, GRAVE),
    (34022970, GRAVE),
    (8633261, DECK),
    (35844557, DECK),
    (72238166, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_type(duel, TYPE_XYZ) || c.is_set_card(duel, &[SET_RYZEAL]))
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

pub fn attachfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL])
        && duel.is_existing_matching_card(|xc, duel| xyzfilter(xc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, None)
}

pub fn xyzfilter(c: CardId, duel: &Duel, mc: CardId, tp: u8) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_rank(duel, &[4]) && c.is_faceup(duel) && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

pub fn attach(duel: &mut Duel, e: EffectId, tp: u8, mc: CardId) -> Script<()> {
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let Some(xyzc) = duel
        .select_matching_card(tp, |c, duel| xyzfilter(c, duel, mc, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if !xyzc.is_immune_to_effect(duel, e) {
        duel.break_effect()?;
        duel.overlay(xyzc, mc)?;
    }
    Ok(())
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(|c, duel| attachfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        if let Some(mc) = duel
            .select_matching_card(tp, |c, duel| attachfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
            .get_first()
        {
            attach(duel, e, tp, mc)?;
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !(c.is_type(duel, TYPE_XYZ) && c.is_rank(duel, &[4])));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}
