// Spatial Trunade
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 2729965;

// Monsters on both fields to return, with Spatial Trunade in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE), (26202165, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            CardId::is_able_to_hand,
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(a.e.get_handler(duel).into()),
        ));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_EITHER, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let mut g = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 2, exc)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? <= 0 {
        return Ok(());
    }
    g.keep_matching(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if !g.is_empty() {
        let ct = g.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
        if ct > 0 {
            duel.shuffle_hand(tp)?;
        }
        if g.len() > ct {
            duel.shuffle_hand(1 - tp)?;
        }
        duel.break_effect()?;
        place_on_deck_bottom(duel, tp, ct)?;
        place_on_deck_bottom(duel, 1 - tp, g.len() - ct)?;
    }
    Ok(())
}

fn place_on_deck_bottom(duel: &mut Duel, p: u8, ct: usize) -> Script<()> {
    if ct == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_able_to_deck, p, LOCATION_HAND, 0, None);
    if g.len() < ct {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, p, HINTMSG_TODECK);
    let sg = g.select(duel, p, ct as u8, ct as u8, None)?;
    if sg.is_empty() {
        return Ok(());
    }
    let sort_ct = duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    if sort_ct > 1 {
        duel.sort_deckbottom(p, p, sort_ct as u16)?;
    }
    Ok(())
}
