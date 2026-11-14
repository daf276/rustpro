// Advanced Heraldry Art
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE};

pub const LISTED_SERIES: &[u16] = &[SET_HERALDIC_BEAST];
// Heraldic Beasts (Leo, Basilisk, Eale) in the Graveyard and Rank 4 Xyz (Number 18, Number 69) to summon with them.
pub const SUPPORT: &[(u32, u16)] = &[
    (82293134, GRAVE),
    (56921677, GRAVE),
    (82315772, GRAVE),
    (23649496, EXTRA),
    (2407234, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(23649496)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_HERALDIC_BEAST])
        && c.is_can_be_effect_target(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn xyzfilter(c: CardId, duel: &Duel, mg: CardSet, tp: Option<u8>) -> bool {
    c.is_xyz_summonable_full(duel, None, Some(mg), 2, 2) && tp.is_none_or(|tp| duel.get_location_count_from_ex(tp, tp, Some(mg), c) > 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let mg = duel.get_matching_group(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(mg.len() >= 2
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_matching_card(|c, duel| xyzfilter(c, duel, mg, None), tp, LOCATION_EXTRA, 0, 1, None));
    }
    let exg = duel.get_matching_group(|c, duel| xyzfilter(c, duel, mg, None), tp, LOCATION_EXTRA, 0, None);
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        let res = sg.len() == 2 && exg.is_exists(duel, |c, duel| c.is_xyz_summonable_full(duel, None, Some(sg), 2, 2), 1, None);
        (res, false)
    };
    let sg = aux::select_unselect_group(duel, mg, 2, 2, &rescon, tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(sg), 2, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_target_cards(e).unwrap_or_default().filter(
        duel,
        |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        None,
    );
    if g.len() != 2 || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? != 2 {
        return Ok(());
    }
    duel.break_effect()?;
    let xyzg = duel.get_matching_group(|c, duel| xyzfilter(c, duel, g, Some(tp)), tp, LOCATION_EXTRA, 0, None);
    if !xyzg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let Some(xyz) = xyzg.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        duel.xyz_summon_full(tp, xyz, Some(g), None, 2, 2)?;
    }
    Ok(())
}
