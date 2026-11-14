// Tri-Brigade Revolt
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, REMOVED, SZONE};

const ID: u32 = 40975243;

pub const LISTED_SERIES: &[u16] = &[SET_TRI_BRIGADE];
// Set copies, Tri-Brigade Kitt, Nervall and Kerass in the GY and banished, and Tri-Brigade Link Monsters in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (56196385, GRAVE),
    (14816857, GRAVE),
    (50810455, GRAVE),
    (56196385, REMOVED),
    (47163170, EXTRA),
    (26847978, EXTRA),
    (99726621, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter2(c: CardId, duel: &Duel, fg: CardSet, minmat: u16, maxmat: u16) -> bool {
    c.is_set_card(duel, &[SET_TRI_BRIGADE]) && c.is_link_summonable_full(duel, None, Some(fg), minmat, maxmat)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> bool {
    let n = sg.len() as u16;
    duel.get_matching_group_count(|c, duel| spfilter2(c, duel, sg, n, n), tp, LOCATION_EXTRA, 0, None) > 0
}

fn filtercheck(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_link_material(duel, None, None)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && (c.is_location(duel, LOCATION_GRAVE) || (c.is_location(duel, LOCATION_REMOVED) && c.is_faceup(duel)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let fg = duel.get_matching_group(
        |c, duel| filtercheck(c, duel, e, tp),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        None,
    );
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    if chk == 0 {
        return Ok(ft > 0
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count_from_ex(tp, tp, None, TYPE_LINK) > 0
            && aux::select_unselect_group_check(duel, fg, 1, ft as usize, &|duel, sg, _, _| (rescon(duel, sg, tp), false)));
    }
    let locations = LOCATION_GRAVE | LOCATION_REMOVED | LOCATION_EXTRA;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = aux::necro_valley_filter(|c, duel| filtercheck(c, duel, e, tp));
    let fg = duel.get_matching_group(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    if ft < 1 || !duel.is_player_can_special_summon_count(tp, 2) {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let rescon = |duel: &Duel, sg, _, _| (rescon(duel, sg, tp), false);
    let g = aux::select_unselect_group_full(duel, fg, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON, Some(&rescon), false)?;
    if g.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for tc in g {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    let n = g.len() as u16;
    let tg = duel.get_matching_group(|c, duel| spfilter2(c, duel, g, n, n), tp, LOCATION_EXTRA, 0, None);
    if !tg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = tg.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.link_summon_full(tp, sc, None, Some(g), n, n)?;
        }
    }
    Ok(())
}
