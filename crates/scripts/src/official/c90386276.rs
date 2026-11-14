// Sinful Spoils Sanctification
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, PZONE, REMOVED, SZONE};

const ID: u32 = 90386276;

pub const LISTED_SERIES: &[u16] = &[SET_DIABELL];

// Set, with Diabell monsters (Diabellze the White Witch) in the GY and banished, and Pendulum Monsters (Dragonpulse Magician)
// in the Pendulum Zones to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (60145298, GRAVE),
    (60145298, REMOVED),
    (60145298, HAND),
    (15146890, PZONE),
    (15146890, PZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn diabellspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DIABELL]) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster_card(duel) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        if !chkc.is_controler(duel, tp) {
            return Ok(false);
        }
        match e.get_label(duel) {
            1 => return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && diabellspfilter(chkc, duel, e, tp)),
            2 => return Ok(chkc.is_location(duel, LOCATION_STZONE) && spfilter(chkc, duel, e, tp)),
            _ => {}
        }
    }
    let mzone_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let b1 = mzone_chk
        && duel.is_existing_matching_card(
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        )
        && duel.is_existing_target(
            |c, duel| diabellspfilter(c, duel, e, tp),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            None,
        );
    let b2 = mzone_chk && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_STZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    let mut g = None;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if op == Some(1) {
        e.set_category(duel, CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON);
        let filter = |c, duel: &Duel| diabellspfilter(c, duel, e, tp);
        g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_STZONE, 0, 1, 1, None)?;
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        let tc = duel.get_first_target();
        if duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0
            && let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
        {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == 2
        && let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
