// Brain Controller
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 44265115;
const BRAIN_CONTROL: u32 = 87910978;

pub const LISTED_NAMES: &[u32] = &[BRAIN_CONTROL];

// In the hand and on the field, with Brain Control to pay Life Points for, and as Synchro material with Petit Angel (Level 3) for
// Armory Arm.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (BRAIN_CONTROL, HAND),
    (BRAIN_CONTROL, DECK),
    (BRAIN_CONTROL, DECK),
    (26202165, MZONE | OPPONENT),
    (38142739, MZONE),
    (38142739, MZONE),
    (29071332, EXTRA),
    (29071332, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_PAY_LPCOST);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |_, a| a.ep == a.tp);
    e3.set_target(duel, lvtg);
    e3.set_operation(duel, lvop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_BE_MATERIAL);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
    });
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, BRAIN_CONTROL) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.has_level(duel) && chkc.is_faceup(duel));
    }
    let filter = aux::faceup_filter(CardId::has_level);
    if chk == 0 {
        return Ok(duel.is_existing_target(&filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, &filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
    let level = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_level(duel));
    let lv = duel.announce_number(tp, &(1..=8).filter(|&n| n != u64::from(level)).collect::<Vec<_>>())?;
    duel.set_target_param(lv as u32);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, 1, tp, lv as i32);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let lv: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)? {
        c.negate_effects(duel, c, RESET_EVENT | RESETS_STANDARD, false);
    }
    duel.special_summon_complete()?;
    Ok(())
}
