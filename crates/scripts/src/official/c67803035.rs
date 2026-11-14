// Reincarnation of the Unchained
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 67803035;

pub const LISTED_SERIES: &[u16] = &[SET_UNCHAINED];
pub const LISTED_NAMES: &[u32] = &[27412542, ID];

// In the hand with Abomination's Prison in the Deck to search, then on the field with Unchained cards in the Graveyard to shuffle
// and a monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (27412542, DECK),
    (27412542, DECK),
    (26236560, GRAVE),
    (31588572, GRAVE),
    (53624265, GRAVE),
    (26236560, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_UNCHAINED]) && c.is_spell(duel) && !c.is_code(duel, ID)) || c.is_code(duel, 27412542))
        && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_UNCHAINED]) && !c.is_code(duel, ID) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_TODECK, g, count as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty() && duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 {
        duel.shuffle_deck(tp)?;
        let c = e.get_handler(duel);
        let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
        let returned_count =
            duel.get_operated_group()
                .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
        if returned_count > 0
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, exc)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, returned_count as u8, exc)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
