// Susurrus of the Sinful Spoils
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 62173132;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];

const DIABELLZE: u32 = 60145298;

// In the hand to activate with Illusions on the field and in the GY (Diabellze the White Witch, Mirror Swordknight), and
// Diabellze in the hand to send it to the GY as a cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (DIABELLZE, HAND),
    (DIABELLZE, HAND),
    (DIABELLZE, HAND),
    (DIABELLZE, MZONE),
    (28954097, MZONE),
    (28954097, GRAVE),
    (28954097, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rttg);
    e1.set_operation(duel, rtop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_reason(duel, REASON_COST) && a.re.is_some_and(|re| re.is_activated(duel) && re.is_monster_effect(duel))
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn rtfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER)
        && c.is_faceup(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && (c.is_able_to_extra(duel) || c.is_able_to_hand(duel))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && rtfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| rtfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(
                    |c, duel| spfilter(c, duel, e, tp),
                    tp,
                    LOCATION_GRAVE | LOCATION_REMOVED,
                    0,
                    1,
                    None,
                ),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, |c, duel| rtfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn rtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && (tc.is_type(duel, TYPE_EXTRA)
            && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
            && tc.is_location(duel, LOCATION_EXTRA)
            || duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0 && tc.is_location(duel, LOCATION_HAND))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

pub fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
