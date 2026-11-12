// Virtual World Mai-Hime - Lulu
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};
use engine::constants::MSG_SELECT_YESNO;

const ID: u32 = 49088914;
const KEY: u32 = TYPE_MONSTER + TYPE_SPELL + TYPE_TRAP;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand, with Virtual World Kyubi - Shenshen (a monster) and Virtual World Gate - Qinglong (a Spell) on the field as the
// target, and Qinglong, Virtual World Gate - Xuanwu (a Trap) in the Deck to send and to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (92519087, MZONE),
    (50275295, SZONE),
    (50275295, DECK),
    (50275295, DECK),
    (18249921, DECK),
    (18249921, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_SELECT_YESNO)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

pub fn togravefilter(c: CardId, duel: &Duel, ctype: u32) -> bool {
    c.is_set_card(duel, &[SET_VIRTUAL_WORLD]) && !c.is_type(duel, ctype & KEY) && c.is_able_to_grave(duel)
}

pub fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let ctype = c.get_type(duel);
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_VIRTUAL_WORLD])
        && duel.is_existing_matching_card(|c, duel| togravefilter(c, duel, ctype), tp, LOCATION_DECK, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c: CardId, duel: &Duel| tgfilter(c, duel, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

pub fn cannot_special_summon(duel: &mut Duel, c: CardId, tp: u8, id: u32) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !(c.is_level_above(duel, 3) || c.is_rank_above(duel, 3))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(id, 1)));
}

fn tohandfilter(c: CardId, duel: &Duel, type1: u32, type2: u32) -> bool {
    c.is_set_card(duel, &[SET_VIRTUAL_WORLD])
        && !c.is_type(duel, type1 & KEY)
        && !c.is_type(duel, type2 & KEY)
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    cannot_special_summon(duel, c, tp, ID);
    let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let ctype = tc.get_type(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| togravefilter(c, duel, ctype), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_grave(g, REASON_EFFECT)? > 0 {
        let ogc = duel.get_operated_group().get_first();
        if ogc.is_some_and(|ogc| ogc.is_location(duel, LOCATION_GRAVE))
            && c.is_relate_to_effect(duel, e)
            && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        {
            let gtype = g.get_first().map_or(0, |gc| gc.get_type(duel));
            let gth = duel.get_matching_group(|c, duel| tohandfilter(c, duel, ctype, gtype), tp, LOCATION_DECK, 0, None);
            if !gth.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                duel.break_effect()?;
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
                let sg = gth.select(duel, tp, 1, 1, None)?;
                duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, sg)?;
            }
        }
    }
    Ok(())
}
