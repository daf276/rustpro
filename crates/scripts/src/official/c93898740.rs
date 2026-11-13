// Unchained Syncretism
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, SZONE};

use super::c27412542::{spcon, spop, sptg};

const ID: u32 = 93898740;

// Activated from the hand with Unchained Soul of Rage and Aruha on the field and Unchained Abomination in the Extra Deck to Link Summon,
// or Set and destroyed by the opponent's Mystical Space Typhoon with Aruha and Sarama in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (67680512, MZONE),
    (26236560, MZONE),
    (29479265, EXTRA),
    (29479265, EXTRA),
    (26236560, DECK),
    (31588572, DECK),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(29479265),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, linksumtg);
    e1.set_operation(duel, linksumop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn unchainedsoulfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_UNCHAINED_SOUL]) && c.is_link_monster(duel) && c.is_controler(duel, tp)
}

fn linkfilter(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_link_summonable_full(duel, None, Some(mg), 2, 2)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> bool {
    sg.is_exists(duel, |c, duel| unchainedsoulfilter(c, duel, tp), 1, None)
        && duel.is_existing_matching_card(|c, duel| linkfilter(c, duel, sg), tp, LOCATION_EXTRA, 0, 1, None)
}

fn linksumtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_target_group(
        aux::faceup_filter(|c, duel| c.is_can_be_link_material(duel, None, None)),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    let rescon = |duel: &Duel, sg, _, _| (rescon(duel, sg, tp), false);
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let tg = aux::select_unselect_group_full(duel, g, 2, 2, &rescon, tp, HINTMSG_TARGET, None, false)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn linkmatimmunefilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_can_be_link_material(duel, None, None) && !c.is_immune_to_effect(duel, e)
}

fn linksumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel
        .get_target_cards(e)
        .unwrap_or_default()
        .filter(duel, |c, duel| linkmatimmunefilter(c, duel, e), None);
    if tg.len() == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| linkfilter(c, duel, tg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(link) = g.get_first() {
            duel.link_summon_full(tp, link, Some(tg), Some(tg), 2, 2)?;
        }
    }
    Ok(())
}
