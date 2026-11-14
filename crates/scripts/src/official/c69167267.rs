// Basal Rose Shoot
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT, SZONE};

const ID: u32 = 69167267;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_BLACK_ROSE_DRAGON];
pub const LISTED_SERIES: &[u16] = &[SET_ROSE_DRAGON];
// Rose Dragons (Red Rose, White Rose) in the hand and GY to summon, Black Rose Dragon in the GY for a set copy the
// opponent's Mystical Space Typhoons destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, SZONE),
    (26118970, HAND),
    (12213463, GRAVE),
    (CARD_BLACK_ROSE_DRAGON, GRAVE),
    (5318639, SZONE | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sstg);
    e1.set_operation(duel, ssop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn ssfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROSE_DRAGON]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| ssfilter(c, duel, e, tp), tp, LOCATION_GRAVE | LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn ssop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| ssfilter(c, duel, e, tp));
    if let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_HAND, 0, 1, 1, None)?
        .get_first()
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_previous_position(duel, POS_FACEDOWN)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_BLACK_ROSE_DRAGON) || c.lists_code(duel, CARD_BLACK_ROSE_DRAGON))
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(spfilter(chkc, duel, e, tp) && chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED));
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(&filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_target_card(tc);
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        Some(tc),
        1,
        tp,
        (LOCATION_GRAVE | LOCATION_REMOVED) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_target_cards(a.e).and_then(|g| g.get_first());
    if duel.get_location_count(a.tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if let Some(tc) = tc {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
