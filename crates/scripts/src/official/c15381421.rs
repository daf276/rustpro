// Starliege Seyfert
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND, MZONE};

const ID: u32 = 15381421;

// Copies on the field and in the Graveyard, with Blue-Eyes White Dragons to send from the hand and field, to search and to recover.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg2);
    e2.set_operation(duel, thop2);
    c.register_effect(duel, e2);
}

fn thcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON)
        && c.get_original_level(duel) > 0
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_level(duel, &[lv]) && c.is_able_to_hand(duel)
}

fn levels(duel: &Duel, sg: CardSet) -> u32 {
    sg.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel))) as u32
}

fn thcheck(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let lv = levels(duel, sg);
    (
        !sg.is_empty() && duel.is_existing_matching_card(|c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, None),
        false,
    )
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(thcfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| thcheck(duel, sg, tp);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 1, g.len(), &rescon));
    }
    let sg = aux::select_unselect_group_full(duel, g, 1, g.len(), &rescon, tp, HINTMSG_TOGRAVE, Some(&rescon), false)?;
    let lv = levels(duel, sg);
    e.set_label(duel, i64::from(lv));
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let lv = a.e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT + ATTRIBUTE_DARK)
        && c.is_level(duel, &[8])
        && c.is_able_to_hand(duel)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter2(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter2, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter2, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
