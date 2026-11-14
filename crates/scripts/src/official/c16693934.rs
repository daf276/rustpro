// Quadogmatika Beast
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 16693934;

// The Lua lists the Dogmatika setcode as a name.
pub const LISTED_NAMES: &[u32] = &[SET_DOGMATIKA as u32];
// In the hand with monsters in the Graveyard to summon and Extra Deck monsters (Battle Shogun, Shi En) to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (2511717, GRAVE),
    (44686185, GRAVE),
    (74752631, EXTRA),
    (29981921, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| tgfilter(sc, duel, c.get_attack(duel)),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn tgfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_monster(duel)
        && (c.is_location(duel, LOCATION_EXTRA) || c.is_set_card(duel, &[SET_DOGMATIKA]))
        && c.is_able_to_grave(duel)
        && c.is_attack_above(duel, atk)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let atk = tc.get_attack(duel);
        let sc = duel.select_matching_card(
            tp,
            |c, duel| tgfilter(c, duel, atk),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        if let Some(sc) = sc.get_first()
            && duel.send_to_grave(sc, REASON_EFFECT)? > 0
            && sc.is_location(duel, LOCATION_GRAVE)
        {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    let count = if duel.is_turn_player(tp) { 2 } else { 1 };
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, count);
    duel.register_effect(e1, tp);
    Ok(())
}
