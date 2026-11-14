// Madolche Queen Tiaramisu
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 37164373;

pub const LISTED_SERIES: &[u16] = &[SET_MADOLCHE];
// In the Extra Deck, Xyz Summoned with Madolche Chouxvalier, with Madolche Puddingcess in the Graveyard and the opponent's cards on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (75363626, MZONE),
    (75363626, MZONE),
    (75363626, MZONE),
    (74641045, GRAVE),
    (74641045, GRAVE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(madolche_filter);
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
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

pub fn madolche_filter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_MADOLCHE], Some(xyz), sumtype, tp)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MADOLCHE]) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    let ct = g.filter_count(
        duel,
        |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_HAND | LOCATION_EXTRA),
        None,
    );
    let dg = duel.get_matching_group(CardId::is_able_to_deck, tp, 0, LOCATION_ONFIELD, None);
    if ct > 0 && !dg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let rg = dg.select(duel, tp, 1, ct as u8, None)?;
        duel.hint_selection(rg);
        duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
