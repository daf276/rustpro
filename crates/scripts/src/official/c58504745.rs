// Brotherhood of the Fire Fist - Cardinal
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 58504745;

// In the Extra Deck, Xyz Summoned from Level 4 Fire Fists (Boar, Buffalo), with Fire Fists in the Graveyard and Sangan on the opponent's side to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (66762372, MZONE),
    (66762372, MZONE),
    (92572371, MZONE),
    (92572371, MZONE),
    (66762372, GRAVE),
    (92572371, GRAVE),
    (26202165, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_FIRE_FIST], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
        && (c.is_set_card(duel, &[SET_FIRE_FIST]) || c.is_set_card(duel, &[SET_FIRE_FORMATION]))
        && c.is_able_to_deck(duel)
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel)) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter1, tp, LOCATION_GRAVE | LOCATION_ONFIELD, 0, 2, None)
            && duel.is_existing_target(filter2, tp, 0, LOCATION_GRAVE | LOCATION_ONFIELD, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let mut g1 = duel
        .select_target(tp, filter1, tp, LOCATION_GRAVE | LOCATION_ONFIELD, 0, 2, 2, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g2 = duel
        .select_target(tp, filter2, tp, 0, LOCATION_GRAVE | LOCATION_ONFIELD, 2, 2, None)?
        .unwrap_or_default();
    g1.merge(g2);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g1), 4, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}
