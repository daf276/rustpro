// Dramatic Snake-Eye Chase
use super::c90241276::treat_as_continuous_spell;
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 26700718;
const DIABELLSTAR_THE_BLACK_WITCH: u32 = 72270339;

// In the hand and the Graveyard, with Diabellstar the Black Witch in the Deck and Graveyard to place and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (DIABELLSTAR_THE_BLACK_WITCH, DECK),
    (DIABELLSTAR_THE_BLACK_WITCH, DECK),
    (DIABELLSTAR_THE_BLACK_WITCH, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8, hand_chk: bool) -> bool {
    if !(c.is_monster(duel) && c.is_set_card(duel, &[SET_DIABELLSTAR]) && !c.is_forbidden(duel)) {
        return false;
    }
    let owner = c.get_owner(duel);
    let mut ft = duel.get_location_count(owner, LOCATION_SZONE);
    if hand_chk && owner == tp {
        ft -= 1;
    }
    ft > 0 && c.check_unique_on_field(duel, owner)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let hand_chk = a.e.get_handler(duel).is_location(duel, LOCATION_HAND);
    if chk == 0 {
        let filter = |c, duel: &Duel| plfilter(c, duel, tp, hand_chk);
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = |c, duel: &Duel| plfilter(c, duel, tp, false);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(filter),
        tp,
        LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.move_to_field(tc, tp, tc.get_owner(duel), LOCATION_SZONE, POS_FACEUP, true)?
    {
        treat_as_continuous_spell(duel, a.e.get_handler(duel), tc);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_original_type(duel, TYPE_MONSTER)
        && c.is_continuous_spell(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_SZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_SZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
