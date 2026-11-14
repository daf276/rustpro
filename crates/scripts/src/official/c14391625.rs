// Visas Samsara
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 14391625;

pub const LISTED_SERIES: &[u16] = &[SET_VISAS];
pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST];

// In the hand with Visas Starfrost on the field and in the Graveyard and Visas Amritara (a Synchro, which returns to the Extra Deck) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (56099748, MZONE), (56099748, GRAVE), (821049, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_VISAS_STARFROST as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_NONTUNER);
    e3.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e3);
}

fn tdfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_VISAS])
        && c.is_monster(duel)
        && c.is_able_to_deck(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let loc = LOCATION_MZONE | LOCATION_REMOVED | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, loc) && tdfilter(chkc, duel, tp));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(|c, duel| tdfilter(c, duel, tp), tp, loc, 0, 1, None));
    }
    let ct = duel.get_matching_group_count(|c, duel| tdfilter(c, duel, tp), tp, loc, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, |c, duel| tdfilter(c, duel, tp), tp, loc, 0, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn visasfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_VISAS]) && c.is_monster(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default().filter(duel, visasfilter, None);
    if g.is_empty() || duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let og = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if og.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let classes = og.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
        c.update_attack(duel, 400 * classes as i32, None, None, None);
    }
    Ok(())
}
