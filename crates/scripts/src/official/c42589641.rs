// Stellarknight Triverr
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 42589641;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT];
// Xyz Summoned from the Extra Deck with three Level 4 "tellarknight" monsters (Satellarknight Deneb, Unukalhai), with another in the
// Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (75878039, MZONE),
    (75878039, MZONE),
    (1050186, MZONE),
    (1050186, MZONE),
    (1050186, GRAVE),
    (75878039, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, lc, sumtype, tp| {
        duel.get_flag_effect(c.get_controler(duel), ID) == 0 && c.is_set_card_full(duel, &[SET_TELLARKNIGHT], Some(lc), sumtype, tp)
    });
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, regcon);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e2.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, regcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, hdtg);
    e4.set_operation(duel, hdop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e5.set_condition(duel, spcon);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e5);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (mut p1, mut p2) = (false, false);
    for tc in a.eg.unwrap_or_default() {
        if !tc.is_set_card(duel, &[SET_TELLARKNIGHT]) {
            if tc.get_summon_player(duel) == 0 {
                p1 = true;
            } else {
                p2 = true;
            }
        }
    }
    if p1 {
        duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    if p2 {
        duel.register_flag_effect(1, ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_TELLARKNIGHT]));
    duel.register_effect(e1, tp);
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, tgp: u8) -> bool {
    st & SUMMON_TYPE_XYZ != SUMMON_TYPE_XYZ || duel.get_flag_effect(tgp, ID) == 0
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(
        CardId::is_able_to_hand,
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(
        CardId::is_able_to_hand,
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}

fn hdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(1 - tp, LOCATION_HAND, 0) != 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(1 - tp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, 1 - tp, 1);
    duel.send_to_grave(sg, REASON_EFFECT)?;
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_count(duel) > 0 && c.get_previous_location(duel) == LOCATION_MZONE
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
