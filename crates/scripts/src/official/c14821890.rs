// Swordsoul Blackout
use super::c93490856::{summon_token, token_stats};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 14821890;

pub const LISTED_NAMES: &[u32] = &[TOKEN_SWORDSOUL];

// Mystery Shell Dragon (a Wyrm) on the field and on the opponent's to destroy three cards, and Pot of Desires to banish this card
// from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (18108166, MZONE),
    (18108166, MZONE | OPPONENT),
    (18108166, MZONE | OPPONENT),
    (35261759, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(TOKEN_SWORDSOUL)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_effect_target(duel, e)
        && (c.is_controler(duel, 1 - tp) || (c.is_faceup(duel) && c.is_monster(duel) && c.is_race(duel, RACE_WYRM)))
}

fn desrescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let own = sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    (own == 1, own > 1)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| {
            desrescon(duel, sg, tp)
        }));
    }
    let dg = aux::select_unselect_group(duel, g, 3, 3, &|duel, sg, _, _| desrescon(duel, sg, tp), tp, HINTMSG_DESTROY)?;
    duel.set_target_card(dg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_monster(tp, TOKEN_SWORDSOUL, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if tktg(duel, a, 0, None)? {
        summon_token(duel, a.e.get_handler(duel), a.tp)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}
