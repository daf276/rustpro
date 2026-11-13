// Levia-Dragon of Atlantis - Daedalus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 64603351;
const SEARCH_COUNT: u8 = 0;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON];

// On the field to search Daedalus Maelstrom (not Level 7) from the Deck, with a second copy in the hand to summon with the Umi it
// becomes, and the opponent's Sangan to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (CARD_UMI, HAND),
    (64233143, DECK),
    (64233143, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, CARD_UMI as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, thcost);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON) || c.is_code(duel, CARD_UMI)),
        a.tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_UMI) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON) && !c.is_level(duel, &[7]) && c.is_able_to_hand(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thcostfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    let max_count = duel.get_matching_group_count(thfilter, tp, LOCATION_DECK, 0, None).min(3);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, thcostfilter, tp, LOCATION_ONFIELD, 0, 1, max_count as u8, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    a.e.set_chain_data(duel, SEARCH_COUNT, g.len() as i64);
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let search_count = a.e.get_chain_data(duel, SEARCH_COUNT).unwrap_or(0) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, search_count, search_count, None)?;
    if g.len() == usize::from(search_count) && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        if duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let sg = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
            if sg.is_empty() {
                return Ok(());
            }
            duel.hint_selection(sg);
            duel.break_effect()?;
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
