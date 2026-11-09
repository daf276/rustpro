// Kewl Tune Remix
use super::c15665977::Procedure;
use super::c65961304::valcheck;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 88170262;

pub const LISTED_NAMES: &[u32] = &[16509007];
pub const MATERIAL: &[u32] = &[16509007];

// In the Extra Deck, Synchro Summoned with Kewl Tune Mix and Tuners on the field, and Tuners in the Graveyard to
// recover on the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (16509007, MZONE),
    (14558127, MZONE),
    (14558127, GRAVE),
    (16509007, GRAVE),
    (89392810, GRAVE),
    (14558127, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure<16509007>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1500);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_TUNER));
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, thsptg);
    e2.set_operation(duel, thspop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_MATERIAL_CHECK);
    e4.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e4);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_TUNER)
        && !c.is_type(duel, TYPE_SYNCHRO)
        && (c.is_able_to_hand(duel) || c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
}

fn validselection(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && sg.is_exists(duel, CardId::is_able_to_hand, 1, Some(c.into()))
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (sg.is_exists(duel, |c, duel| validselection(c, duel, e, tp, sg), 1, None), false)
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| thspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(g.len() >= 2
            && duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synchfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_synchro_summonable(duel, None, None)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let thsp_g = duel.get_matching_group(|c, duel| thspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if thsp_g.len() < 2 {
        return Ok(());
    }
    let g = aux::select_unselect_group(
        duel,
        thsp_g,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        aux::stringid(ID, 1),
    )?;
    if g.len() < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(thc) = g
        .filter_select(duel, tp, |c, duel| validselection(c, duel, e, tp, g), 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let sc = g.difference(thc.into()).get_first();
    if duel.send_to_hand(thc, PLAYER_NONE, REASON_EFFECT)? > 0 && thc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, thc)?;
        duel.shuffle_hand(tp)?;
        if let Some(sc) = sc
            && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        {
            let synchg = duel.get_matching_group(synchfilter, tp, LOCATION_EXTRA, 0, None);
            if synchg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                return Ok(());
            }
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            if let Some(synchc) = synchg.select(duel, tp, 1, 1, None)?.get_first() {
                duel.synchro_summon(tp, synchc, None, None)?;
            }
        }
    }
    Ok(())
}
