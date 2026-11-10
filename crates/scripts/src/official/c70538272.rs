// Filia Regis
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 70538272;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp) && c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp)
        }),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp) && c.is_level_above(duel, 7)
        }),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Darkfire Dragon and Seiyaryu, a copy in the Graveyard to Special
// Summon in the opponent's Battle Phase by returning Seiyaryu, and Sangan of the opponent's to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (17881964, MZONE),
    (6740720, HAND),
    (6740720, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn rethandfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_able_to_hand(duel) && c.is_faceup(duel)
}

fn rmfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_remove(duel)
        && (c.is_location(duel, LOCATION_GRAVE) || duel.is_existing_matching_card(rethandfilter, tp, LOCATION_MZONE, 0, 1, None))
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE)
            && chkc.is_controler(duel, 1 - tp)
            && rmfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| rmfilter(c, duel, tp), tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(
        tp,
        |c, duel| rmfilter(c, duel, tp),
        tp,
        0,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    if g.and_then(|g| g.get_first())
        .is_some_and(|tc| tc.is_location(duel, LOCATION_ONFIELD))
    {
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_MZONE as i32);
    }
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && tc.is_previous_location(duel, LOCATION_ONFIELD)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let g = duel.select_matching_card(tp, rethandfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0
        && c.is_race(duel, RACE_DRAGON)
        && (c.is_able_to_hand(duel) || c.is_able_to_extra(duel))
        && c.is_faceup(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| thfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let tc = duel
        .select_matching_card(tp, |c, duel| thfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND | LOCATION_EXTRA)
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
