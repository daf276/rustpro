// Azamina Debtors
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 20934683;

// A copy in the Graveyard to banish and Elzette (an Azamina monster) there to Special Summon, WANTED: Seeker of Sinful Spoils in
// the hand to send, and the Azamina Mu Rcielago Fusion Monster in the Extra Deck, with Jongleur-Ghoul Illusionist (an Illusion
// Normal Monster) and Mystical Elf (a LIGHT Spellcaster) in the hand as its materials.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, HAND),
    (38648860, GRAVE),
    (80845034, HAND),
    (80845034, HAND),
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
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase_full(None));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn sinfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SINFUL_SPOILS]) && c.is_able_to_grave(duel)
}

fn exspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
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
        let ct = duel.get_matching_group_count(sinfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, None) as u32;
        return Ok(
            ct > 0 && duel.is_existing_matching_card(|c, duel| exspfilter(c, duel, e, tp, ct * 4 + 3), tp, LOCATION_EXTRA, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_ONFIELD | LOCATION_HAND) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sg = duel.get_matching_group(sinfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = sg.len() as u32 * 4 + 3;
    let g = duel.select_matching_card(tp, |c, duel| exspfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, tc)?;
    let ct = (tc.get_level(duel) / 4) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let ssg = sg.select(duel, tp, ct, ct, None)?;
    if ssg.is_empty() {
        return Ok(());
    }
    let fdg = ssg.filter(duel, |c, duel| c.is_facedown(duel) && c.is_on_field(duel), None);
    if !fdg.is_empty() {
        duel.confirm_cards(1 - tp, fdg)?;
    }
    if duel.send_to_grave(ssg, REASON_EFFECT)? > 0 && ssg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None) {
        tc.set_material(duel, None);
        duel.break_effect()?;
        if duel.special_summon(tc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? == 0 {
            return Ok(());
        }
        tc.complete_procedure(duel);
    }
    Ok(())
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_AZAMINA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        // The Lua checks s.filter, which does not exist, so it errors.
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
