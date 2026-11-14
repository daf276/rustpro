// Azamina Determination
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 42201897;

pub const LISTED_SERIES: &[u16] = &[SET_AZAMINA, SET_SINFUL_SPOILS];
// A copy in the Graveyard for its own effect, Sinful Spoils Spells and Traps there to return or Set, and the Azamina Mu Rcielago
// Fusion Monster in the Extra Deck with Jongleur-Ghoul Illusionist and Mystical Elf in the hand as its materials.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (3496543, GRAVE),
    (38511382, GRAVE),
    (73391962, EXTRA),
    (47894537, HAND),
    (15025844, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(73391962),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn sinfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SINFUL_SPOILS]) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_AZAMINA])
        && c.is_type(duel, TYPE_FUSION)
        && c.is_level_above(duel, 4)
        && c.is_level_below(duel, lv)
        && !c.is_public(duel)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.check_fusion_material(duel, None, None, u64::from(PLAYER_NONE))
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ct = duel.get_matching_group_count(sinfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None) as u32;
        return Ok(ct > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, ct * 4 + 3), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sg = duel.get_matching_group(aux::necro_valley_filter(sinfilter), tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = sg.len() as u32 * 4 + 3;
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, tc)?;
    let ct = (tc.get_level(duel) / 4) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let ssg = sg.select(duel, tp, ct, ct, None)?;
    if ssg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(ssg);
    if duel.send_to_deck(ssg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && ssg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), 1, None)
    {
        tc.set_material(duel, None);
        duel.break_effect()?;
        if duel.special_summon(tc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? == 0 {
            return Ok(());
        }
        tc.complete_procedure(duel);
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SINFUL_SPOILS]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && setfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(setfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, setfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_ssetable(duel) && duel.sset(a.tp, tc)? > 0 {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESET_EVENT | RESETS_CANNOT_ACT | RESET_PHASE | PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
