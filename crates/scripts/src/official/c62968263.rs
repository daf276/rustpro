// Galaxy-Eyes Afterglow Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 62968263;

pub const LISTED_SERIES: &[u16] = &[SET_GALAXY_EYES, SET_NUMBER];
pub const LISTED_NAMES: &[u32] = &[CARD_GALAXYEYES_P_DRAGON];

// In the hand and on the field, with Galaxy-Eyes Cipher Dragon Xyz Summoned from two of them, whose detach cost sends one to the
// Graveyard, and Galaxy-Eyes Photon Dragons in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, DECK),
    (CARD_GALAXYEYES_P_DRAGON, DECK),
    (18963306, EXTRA),
    (18963306, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, atcon);
    e2.set_target(duel, attg);
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GALAXY_EYES]));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, a.e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_COST)
        && a.re.is_some_and(|re| re.is_activated(duel) && re.is_active_type(duel, TYPE_XYZ))
        && c.is_previous_location(duel, LOCATION_OVERLAY)
}

fn attfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && !c.is_immune_to_effect(duel, e)
}

fn gepdfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_code(duel, CARD_GALAXYEYES_P_DRAGON)
        && ((ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
            || duel.is_existing_matching_card(|c, duel| attfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, None))
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| gepdfilter(c, duel, e, tp, ft),
            tp,
            LOCATION_DECK | LOCATION_HAND,
            0,
            1,
            None,
        ));
    }
    let battle = i64::from(duel.is_battle_phase());
    e.set_label(duel, battle);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_HAND) as i32);
    Ok(false)
}

fn numbfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_NUMBER])
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(
        tp,
        |c, duel| gepdfilter(c, duel, e, tp, ft),
        tp,
        LOCATION_DECK | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    let spchk = ft > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let attchk = duel.is_existing_matching_card(|c, duel| attfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, None);
    if !(spchk || attchk) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(spchk, aux::stringid(ID, 3)), (attchk, aux::stringid(ID, 4))])?;
    let mut success = false;
    if op == Some(1) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        success = true;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let g = duel.select_matching_card(tp, |c, duel| attfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, 1, None)?;
        if let Some(oc) = g.get_first() {
            success = true;
            duel.hint_selection(oc);
            duel.overlay(oc, tc)?;
        }
    }
    let ng = duel.get_matching_group(numbfilter, tp, LOCATION_MZONE, 0, None);
    if success && e.get_label(duel) == 1 && !ng.is_empty() {
        let c = e.get_handler(duel);
        duel.break_effect()?;
        for sc in ng {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            let attack = sc.get_attack(duel) * 2;
            e1.set_value(duel, attack);
            sc.register_effect(duel, e1);
        }
    }
    Ok(())
}
