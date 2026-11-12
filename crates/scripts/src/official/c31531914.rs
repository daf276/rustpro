// Unchained Ogre Shma
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 31531914;

pub const LISTED_NAMES: &[u32] = &[ID];

// Several in the hand to Normal Summon, a copy on the field and Dark Hole in the opponent's hand to destroy it, with Unchained
// Twins in the Deck and the hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (26236560, DECK),
    (31588572, DECK),
    (53624265, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, lv4sptg);
    e1.set_operation(duel, lv4spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, handdeckspcon);
    e2.set_target(duel, handdecksptg);
    e2.set_operation(duel, handdeckspop);
    c.register_effect(duel, e2);
}

fn unchainedspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_UNCHAINED]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn lv4spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && unchainedspfilter(c, duel, e, tp)
}

fn lv4sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| lv4spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn lv4spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| lv4spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let dg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
            if !dg.is_empty() {
                duel.hint_selection(dg);
                duel.break_effect()?;
                duel.destroy(dg, REASON_EFFECT)?;
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
    e1.set_target_card(duel, |duel, _, c| c.is_race_except(duel, RACE_FIEND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_FIEND));
    Ok(())
}

fn handdeckspcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if !(c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)) {
        return false;
    }
    if c.is_reason(duel, REASON_BATTLE) {
        return true;
    }
    unported!(duel, a.re.is_none());
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    if !duel.is_chain_solving() {
        return !rc.is_code(duel, ID);
    }
    if rc == c {
        !c.is_previous_code_on_field(duel, &[ID])
    } else if rc.is_relate_to_effect(duel, re) && rc.is_faceup(duel) {
        !rc.is_code(duel, ID)
    } else {
        let code1: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CODE);
        let code2: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CODE2);
        code1 != ID && code2 != ID
    }
}

fn handdecksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| unchainedspfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
    )
}

fn handdeckspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| unchainedspfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
        POS_FACEUP,
    )
}
