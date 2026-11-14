// Adamancipator Risen - Leonite
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 47674738;

pub const LISTED_SERIES: &[u16] = &[SET_ADAMANCIPATOR];
// In the Extra Deck, Synchro Summoned with Angel Trumpeter (a Level 4 Tuner) and Doron (Level 2), with Adamancipator monsters
// in the Deck to excavate and Stone Ogre Grotto (Rock) and Charcoal Inpachi (FIRE) in the Graveyard for the quick effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (87979586, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (48519867, DECK),
    (85914562, DECK),
    (11302671, DECK),
    (15023985, GRAVE),
    (15023985, GRAVE),
    (13179332, GRAVE),
    (13179332, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ADAMANCIPATOR]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) >= 5);
    }
    duel.set_target_player(tp);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let mut ac = 5;
    duel.confirm_decktop(p, ac)?;
    let g = duel.get_decktop_group(p, ac);
    if !g.is_empty() && g.is_exists(duel, thfilter, 1, None) && duel.select_yes_no(p, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, p, HINTMSG_ATOHAND);
        let sg = g.filter_select(duel, p, thfilter, 1, 1, None)?;
        duel.disable_shuffle_check(true);
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - p, sg)?;
        duel.shuffle_hand(p)?;
        ac -= 1;
    }
    if ac > 0 {
        duel.move_to_deck_bottom_count(ac, tp)?;
        duel.sort_deckbottom(tp, tp, ac as u16)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && duel.is_existing_matching_card(|c, duel| c.is_attribute(duel, ATTRIBUTE_FIRE), a.tp, LOCATION_GRAVE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}
