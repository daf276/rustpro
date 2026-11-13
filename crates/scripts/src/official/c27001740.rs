// Farm Delivery
use crate::prelude::*;
use crate::testing::{HAND, MZONE, REMOVED};

const ID: u32 = 27001740;

// A Normal Monster on the field (Psychic Kappa) and banished ones (Hyosube, Water Omotics) to shuffle back.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (7892180, MZONE), (2118022, REMOVED), (2483611, REMOVED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_condition(duel, tdcon);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_NORMAL)),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let tg = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, 0, 1, 3, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, tg, tg.map_or(0, |tg| tg.len()) as u8, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.send_to_deck(tg, a.tp, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
