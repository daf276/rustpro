// The Hallowed Azamina
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 94845588;

// In the Graveyard, with Elzette (an Azamina monster) on the field to shuffle.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (38648860, MZONE)];
// No Sinful Spoils card is ported, so nothing is sent to the Graveyard for the Fusion Summon.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn sinfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SINFUL_SPOILS]) && c.is_able_to_grave(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32, g: CardSet) -> bool {
    c.is_set_card(duel, &[SET_AZAMINA])
        && c.is_type(duel, TYPE_FUSION)
        && c.is_level_above(duel, 4)
        && c.is_level_below(duel, lv)
        && !c.is_public(duel)
        && duel.get_location_count_from_ex(tp, tp, Some(g), c) > 0
        && c.check_fusion_material(duel, None, None, u64::from(PLAYER_NONE))
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let g = duel.get_matching_group(sinfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, None);
        let ct = g.len() as u32;
        return Ok(
            ct > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, ct * 4 + 3, g), tp, LOCATION_EXTRA, 0, 1, None)
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
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, lv, sg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, tc)?;
    let ct = (tc.get_level(duel) / 4) as usize;
    let ssg = aux::select_unselect_group(
        duel,
        sg,
        ct,
        ct,
        &|duel, sg, _, _| (duel.get_location_count_from_ex(tp, tp, Some(sg), tc) > 0, false),
        tp,
        HINTMSG_TOGRAVE,
    )?;
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

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_AZAMINA]) && c.is_monster(duel) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_target(tdfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
        && c.is_relate_to_effect(duel, e)
    {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
