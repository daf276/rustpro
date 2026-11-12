// Morph King Stygi-Gel
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 85457355;

// Copies in the hand to Normal Summon, a Level 7 monster on the field to copy the Level of, and Level 2 and 3 Fiends in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (24311372, MZONE),
    (4920010, HAND),
    (7670542, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn lvfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && !c.is_level(duel, &[lv])
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let lv = c.get_level(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && lvfilter(chkc, duel, lv) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| lvfilter(c, duel, lv), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c, duel| lvfilter(c, duel, lv), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first()) {
        let amount = tc.get_level(duel) as i32 * 200;
        duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, amount);
    }
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
    {
        let lv = tc.get_level(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        if c.is_level(duel, &[lv]) {
            duel.recover(tp, lv * 200, REASON_EFFECT)?;
        }
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 0);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_card(duel, |duel, _, c| {
        !c.is_type(duel, TYPE_SYNCHRO) && c.is_location(duel, LOCATION_EXTRA)
    });
    duel.register_effect(e2, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_race(duel, RACE_FIEND)
        && c.has_level(duel)
        && c.is_level_below(duel, lv.wrapping_sub(1))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let lv = a.e.get_handler(duel).get_level(duel);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp, lv), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let lv = e.get_handler(duel).get_level(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, lv), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
