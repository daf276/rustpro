// Overburst Dragon
use super::c12023931::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 15627227;

// In the Extra Deck, Link Summoned from Rokket monsters or on the field with copies, with Rokket monsters in the hand to Special
// Summon, and in the Graveyard to add when it is destroyed in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (25554552, MZONE),
    (67127799, HAND),
    (48355999, HAND),
    (25554552, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_controler(duel, a.tp) && c.is_reason(duel, REASON_BATTLE)
    });
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_faceup(duel) && c.is_link_monster(duel)) {
        return false;
    }
    let zone = c.get_linked_zone_full(duel, tp);
    zone != 0 && duel.is_existing_matching_card(|hc, duel| spfilter2(hc, duel, e, tp, zone), tp, LOCATION_HAND, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && spfilter1(chkc, duel, e, tp) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(lc) = duel.get_first_target() else { return Ok(()) };
    if !(lc.is_relate_to_effect(duel, e) && lc.is_faceup(duel)) {
        return Ok(());
    }
    let zone = lc.get_linked_zone_full(duel, tp);
    if zone == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp, zone), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)?
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3312);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        let fid = c.get_field_id(duel);
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_code(duel, EVENT_PHASE + PHASE_END);
        e2.set_count_limit(duel, 1, 0);
        e2.set_label(duel, fid.into());
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_condition_mut(duel, descon);
        e2.set_operation(duel, desop);
        duel.register_effect(e2, tp);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    if tc.is_some_and(|tc| tc.get_flag_effect_label(duel, ID) != Some(a.e.get_label(duel))) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_ROKKET]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
