// Mirror Resonator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 40159926;

// In the hand and Graveyard to Special Summon after the opponent Synchro Summons Red Dragon Archfiend (Soul Resonator and
// Shapesnatch as materials), and on the field to copy a Level.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, MZONE),
    (62991792, MZONE | OPPONENT),
    (62991792, MZONE | OPPONENT),
    (4035199, MZONE | OPPONENT),
    (4035199, MZONE | OPPONENT),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA | OPPONENT),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA | OPPONENT),
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
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_summon_location(duel, LOCATION_EXTRA)),
        tp,
        0,
        LOCATION_MZONE,
        1,
        None,
    ) && !duel.is_existing_matching_card(|c, duel| c.is_summon_location(duel, LOCATION_EXTRA), tp, LOCATION_MZONE, 0, 1, None)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_original_level(duel) > 0
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && lvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, lvfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let level = tc.get_original_level(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SYNCHRO_LEVEL);
        e1.set_value(duel, level as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        c.register_flag_effect_full(duel, 0, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_CLIENT_HINT, 1, 0, level.into());
    }
    Ok(())
}
