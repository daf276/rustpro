// Hot Red Dragon Archfiend Bane
use super::c9753964::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 36857073;

// In the Extra Deck, Synchro Summoned with Formula Synchron and Red Dragon Archfiend, on the field with another monster to
// release for the one in the Graveyard, and Mono Synchron in the Deck with Effect Veiler (Level 1 Tuners) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (50091196, MZONE),
    (70902743, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (70902743, GRAVE),
    (97268402, GRAVE),
    (56897896, DECK),
    (56897896, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.ep != a.tp);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, aux::TRUE, (1, 1), false, Some(&aux::release_check_mmz), None));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), false, Some(&aux::release_check_mmz), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RED_DRAGON_ARCHFIEND]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter1(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn defense_summonable(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    defense_summonable(c, duel, e, tp)
        && duel.is_existing_matching_card(
            |g, duel| spfilter3(g, duel, e, tp, c.get_level(duel)),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter3(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    defense_summonable(c, duel, e, tp) && c.get_level(duel) == lv
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g1 = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(first) = g1.get_first() else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = first.get_level(duel);
    let g2 = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter3(c, duel, e, tp, lv)),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    g1.merge(g2);
    duel.special_summon(g1, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    Ok(())
}
