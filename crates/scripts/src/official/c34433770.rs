// Sky Striker Special Maneuver - Lemnisgate!
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 34433770;

// In the hand and the GY, with Sky Striker Ace monsters and Spells in the GY to shuffle and Link Monsters in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (26077387, GRAVE),
    (37351133, GRAVE),
    (98338152, GRAVE),
    (52340444, GRAVE),
    (8491308, EXTRA),
    (63288573, EXTRA),
    (26077387, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    ((c.is_set_card(duel, &[SET_SKY_STRIKER_ACE]) && c.is_monster(duel)) || (c.is_set_card(duel, &[SET_SKY_STRIKER]) && c.is_spell(duel)))
        && c.is_able_to_deck(duel)
        && c.is_can_be_effect_target(duel, e)
}

fn rescon(full_mct: usize, full_sct: usize) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        let mct = sg.filter_count(duel, CardId::is_monster, None);
        let sct = sg.len() - mct;
        if mct == sct {
            return (true, false);
        }
        let (rem_mct, rem_sct) = (full_mct as i64 - mct as i64, full_sct as i64 - sct as i64);
        let (mct, sct) = (mct as i64, sct as i64);
        (false, (mct > sct && rem_sct < mct - sct) || rem_mct < sct - mct)
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tdfilter(c, duel, e), tp, LOCATION_GRAVE, 0, None);
    let full_mct = g.filter_count(duel, CardId::is_monster, None);
    let full_sct = g.len() - full_mct;
    let rescon = rescon(full_mct, full_sct);
    if chk == 0 {
        return Ok(full_mct > 0 && full_sct > 0 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let maxc = full_mct.min(full_sct) * 2;
    let tg = aux::select_unselect_group_full(duel, g, 2, maxc, &rescon, tp, HINTMSG_TODECK, Some(&rescon), false)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), tg.len() as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tg) = duel.get_target_cards(e) else { return Ok(()) };
    if !tg.is_empty()
        && duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
    {
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None)
            / 3;
        if ct > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
            let g = duel.select_matching_card(
                tp,
                CardId::is_able_to_hand,
                tp,
                LOCATION_ONFIELD,
                LOCATION_ONFIELD,
                1,
                ct as u8,
                None,
            )?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER]) && c.is_controler(duel, tp) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER_ACE]) && c.is_link_summonable(duel, None, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(spfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, spfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    {
        duel.link_summon(tp, sc, None, None)?;
    }
    Ok(())
}
