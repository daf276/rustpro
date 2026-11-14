// Artmage Masterwork -Succession-
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 37517035;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];
// In hand to Fusion Summon Artmage Diactorus from the Extra Deck with Medius the Pure and Artmage Litera, with Acropolis in a random
// zone for the ATK, and in the Graveyard to shuffle three Artmage cards with different names.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (27184601, EXTRA),
    (CARD_MEDIUS_THE_PURE, HAND),
    (97434754, HAND),
    (97434754, GRAVE),
    (34541940, GRAVE),
    (60946049, GRAVE),
    (74733322, 0),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(27184601), Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Params;

impl FusionSummonParams for Params {
    fn extrafil(duel: &Duel, e: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, Some((matcheck, e.get_handler(duel))))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, None, 1, a.tp, 500);
        }
        Ok(true)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 1
            && let Some(tc) = tc
            && duel.is_existing_matching_card(aux::TRUE, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
        {
            duel.break_effect()?;
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 500);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
        Ok(())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn matcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| c.is_set_card_full(duel, &[SET_ARTMAGE], Some(fc), SUMMON_TYPE_FUSION.into(), tp),
        1,
        None,
    )
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE]) && c.is_able_to_deck(duel) && c.is_can_be_effect_target(duel, e)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| tdfilter(c, duel, e),
        tp,
        LOCATION_GRAVE,
        0,
        Some(e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(g.len() >= 3 && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3);
    }
    let tg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_TODECK)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 3, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
