// Barian's Seal
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 38001022;

// Set with a Number C104 on the field and Umbral Horror monsters in the Graveyard, against the opponent's Raigeki, with a monster to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (49456901, MZONE),
    (2932235, GRAVE),
    (3064783, GRAVE),
    (39053882, GRAVE),
    (76548531, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn negconfilter(c: CardId, duel: &Duel) -> bool {
    if c.is_facedown(duel) {
        return false;
    }
    let xyznumber = c.xyz_number(duel);
    c.is_set_card(duel, &[SET_CXYZ])
        || (c.is_set_card(duel, &[SET_NUMBER_C]) && xyznumber.is_some_and(|number| (101..=107).contains(&number)))
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return false };
    a.rp == 1 - tp
        && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(ev)
        && duel.is_existing_matching_card(negconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn xyzfilter(c: CardId, duel: &Duel, mc: CardId, tp: u8) -> bool {
    c.is_xyz_monster(duel) && c.is_faceup(duel) && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && duel.is_existing_matching_card(|xc, duel| xyzfilter(xc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let filter = aux::necro_valley_filter(|c, duel| attachfilter(c, duel, tp));
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if duel.negate_activation(a.ev as u8)
        && duel.is_existing_matching_card(&filter, tp, locations, locations, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        let Some(mc) = duel
            .select_matching_card(tp, &filter, tp, locations, locations, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(mc);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let Some(xyzc) = duel
            .select_matching_card(tp, |c, duel| xyzfilter(c, duel, mc, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(xyzc);
        if !xyzc.is_immune_to_effect(duel, e) {
            duel.break_effect()?;
            duel.overlay_full(xyzc, mc, true)?;
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_UMBRAL_HORROR]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    if g.len() == 2 {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
