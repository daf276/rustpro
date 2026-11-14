// Magical Musket - Dancing Needle
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE};

const ID: u32 = 66149377;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];
// In the hand to activate with Magical Musketeer Kidbrave on the field, and a card in the GY to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (5230799, MZONE), (5230799, MZONE), (5230799, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MAGICAL_MUSKET])
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_REMOVE, g, count as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
