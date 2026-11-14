// Salamangreat Mole
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 89484053;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];

// Copies in the hand and GY, with Salamangreat Sunlight Wolf and Almiraj (pointing down from the Extra Monster Zone) to Link Summon
// from FIRE Effect Monsters (Link Apple, Jet Synchron), Salamangreat cards in the GY to shuffle and the opponent's Raigeki to clear the
// field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (87871125, EXTRA),
    (87871125, EXTRA),
    (60303245, EXTRA),
    (60303245, EXTRA),
    (7925734, MZONE),
    (9742784, MZONE),
    (7925734, MZONE),
    (9742784, MZONE),
    (26889158, GRAVE),
    (26889158, GRAVE),
    (26889158, GRAVE),
    (14934922, GRAVE),
    (14934922, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_custom_activity_count(ID, a.tp, ACTIVITY_SPSUMMON) > 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(lkfilter));
}

fn lkfilter(duel: &Duel, c: CardId) -> bool {
    !(c.is_link_monster(duel) && c.is_link_summoned(duel))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = aux::get_mm_zones_pointed_to(duel, tp);
    if chk == 0 {
        return Ok(zone > 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = aux::get_mm_zones_pointed_to(duel, tp);
    if zone > 0 && c.is_relate_to_effect(duel, e) {
        duel.special_summon_full(c, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_able_to_deck(duel)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_player_can_draw(tp, 2) && duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 5, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 5, 5, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_TODECK, g, count as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() || duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let og = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if og.is_empty() {
        return Ok(());
    }
    if og.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    duel.break_effect()?;
    duel.draw(tp, 2, REASON_EFFECT)?;
    Ok(())
}
