// Gunkan Suship Uni
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 42377643;
pub const LISTED_NAMES: &[u32] = &[CARD_SUSHIP_SHARI];

// In the hand with a Gunkan Suship Shari (or Ikura) to reveal, a second Uni on the field for the Level change, and Shari in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_SUSHIP_SHARI, HAND),
    (CARD_SUSHIP_SHARI, DECK),
    (CARD_SUSHIP_SHARI, DECK),
    (61027400, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GUNKAN]) && !c.is_public(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    e.set_label_object_group(duel, Some(g));
    duel.set_target_card(g);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    let Some(rc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    if rc.is_code(duel, CARD_SUSHIP_SHARI) {
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(rc.into()), 1, tp, LOCATION_HAND as i32);
    } else {
        duel.set_operation_info(0, CATEGORY_TODECK, Some(rc.into()), 1, 0, 0);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = e.get_label_object_group(duel).unwrap_or_default();
    let c = e.get_handler(duel);
    let Some(rc) = g.get_first() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 && rc.is_relate_to_effect(duel, e)
    {
        if duel.get_location_count(tp, LOCATION_MZONE) > 0
            && rc.is_code(duel, CARD_SUSHIP_SHARI)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        } else if !rc.is_code(duel, CARD_SUSHIP_SHARI) {
            duel.break_effect()?;
            duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GUNKAN]) && c.has_level(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_SUSHIP_SHARI) && c.is_able_to_hand(duel)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let level = u64::from(tc.get_level(duel));
        let lv = duel.announce_number(tp, &(4..=5).filter(|&n| n != level).collect::<Vec<_>>())?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
