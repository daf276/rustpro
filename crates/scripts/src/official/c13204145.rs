// Mimighoul Maker
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 13204145;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// In the hand with Mimighoul Fairy and Archfiend (Flip monsters) in the Deck, and a copy in the Graveyard with Sangan
// face-down on the opponent's field to flip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (43066927, DECK),
    (43066927, DECK),
    (50415441, DECK),
    (50415441, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, revtg);
    e1.set_operation(duel, revop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn revfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_FLIP)
        && !c.is_public(duel)
        && c.is_able_to_hand(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, 1 - tp, 0xff)
}

fn opponent_zones(duel: &Duel, tp: u8) -> i32 {
    duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff)
}

fn revtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if opponent_zones(duel, tp) == 0 {
            return Ok(false);
        }
        let g = duel.get_matching_group(|c, duel| revfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MIMIGHOUL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn revop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if opponent_zones(duel, tp) == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| revfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let mut sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_SELECT)?;
    if sg.len() != 2 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, sg)?;
    let Some(sc) = sg.random_select(duel, 1 - tp, 1).get_first() else {
        return Ok(());
    };
    if duel.special_summon(sc, 0, tp, 1 - tp, false, false, POS_FACEDOWN_DEFENSE)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(tp, sc)?;
    sg.remove(sc);
    if duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, sg)?;
    duel.shuffle_hand(tp)?;
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let hg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    if hg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let hsg = hg.select(duel, tp, 1, 1, None)?;
    if !hsg.is_empty() {
        duel.break_effect()?;
        duel.special_summon(hsg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(posfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, posfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let pos = (POS_FACEUP_ATTACK | POS_FACEUP_DEFENSE) & !tc.get_position(duel);
        let pos = duel.select_position(a.tp, tc, pos as u8)?;
        duel.change_position(tc, pos)?;
    }
    Ok(())
}
