// Crystron Tristaros
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 99471856;

pub const LISTED_NAMES: &[u32] = &[ID];
// On the field and in the Graveyard, with Crystron Smiger (Level 3) in the Deck to Synchro Summon with Accel Synchron (Level 5)
// in the Extra Deck, and Accel Synchron on the field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (83443619, DECK),
    (83443619, DECK),
    (83443619, DECK),
    (37675907, EXTRA),
    (37675907, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(37675907),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e1.set_target(duel, syncsstg);
    e1.set_operation(duel, syncssop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn crystronfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_CRYSTRON])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|sc, duel| synchfilter(sc, duel, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn synchfilter(c: CardId, duel: &Duel, matc: CardId) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_synchro_summonable(duel, Some(matc.into()), None)
}

fn syncsstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| crystronfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn syncssop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = duel
        .select_matching_card(tp, |c, duel| crystronfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let g = duel.get_matching_group(|sc, duel| synchfilter(sc, duel, tc), tp, LOCATION_EXTRA, 0, None);
        if g.is_empty() {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sc = g.select(duel, tp, 1, 1, None)?.get_first();
        if let Some(sc) = sc {
            duel.synchro_summon(tp, sc, Some(tc.into()), None)?;
        }
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO) && duel.get_mzone_count(tp, Some(c.into())) >= 2
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_CRYSTRON]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, None)
                && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none(),
        );
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if !dg.is_empty() {
        duel.hint_selection(dg);
        if duel.destroy(dg, REASON_EFFECT)? > 0
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, 2, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_MACHINE)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_MACHINE));
    Ok(())
}
