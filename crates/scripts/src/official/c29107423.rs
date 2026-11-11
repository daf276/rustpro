// Ruddy Rose Witch
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 29107423;
const WITCH_OF_THE_BLACK_ROSE: u32 = 17720747;
const RUDDY_ROSE_DRAGON: u32 = 40139997;

pub const LISTED_NAMES: &[u32] = &[WITCH_OF_THE_BLACK_ROSE, CARD_BLACK_ROSE_DRAGON, RUDDY_ROSE_DRAGON];

// On the field and in the Graveyard, with Witch of the Black Rose and Dark Plant (Level 1 Plant Normal Monster) in the Deck
// and Black Rose Dragon in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (WITCH_OF_THE_BLACK_ROSE, DECK),
    (WITCH_OF_THE_BLACK_ROSE, DECK),
    (WITCH_OF_THE_BLACK_ROSE, HAND),
    (13193642, DECK),
    (13193642, DECK),
    (CARD_BLACK_ROSE_DRAGON, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, textg);
    e2.set_operation(duel, texop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, WITCH_OF_THE_BLACK_ROSE)
        && c.is_able_to_hand(duel)
        && duel.is_existing_matching_card(topdeckfilter, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn topdeckfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 3) && c.is_race(duel, RACE_PLANT)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, WITCH_OF_THE_BLACK_ROSE) && c.is_summonable(duel, true, None)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sc = duel
        .select_matching_card(tp, |c, duel| thfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    let Some(sc) = sc else { return Ok(()) };
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let topc = duel
            .select_matching_card(tp, topdeckfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
            .get_first();
        if let Some(topc) = topc {
            duel.shuffle_deck(tp)?;
            duel.move_sequence(topc, 0)?;
            duel.confirm_decktop(tp, 1)?;
            if duel.is_existing_matching_card(sumfilter, tp, LOCATION_HAND, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
                if let Some(sumc) = duel
                    .select_matching_card(tp, sumfilter, tp, LOCATION_HAND, 0, 1, 1, None)?
                    .get_first()
                {
                    duel.summon(tp, sumc, true, None)?;
                }
            }
        }
    }
    Ok(())
}

fn texfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_BLACK_ROSE_DRAGON) || c.is_code(duel, RUDDY_ROSE_DRAGON)) && c.is_faceup(duel) && c.is_able_to_extra(duel)
}

fn textg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(texfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn texop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(texfilter),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
