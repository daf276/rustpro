// Trisborrel Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 3957130;

pub const LISTED_NAMES: &[u32] = &[67526112, 20071842, 38129297];

// In the Extra Deck, Link Summoned from two Rokket monsters, with the three Spells in the Deck and a DARK Dragon in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (25554552, MZONE),
    (68464358, MZONE),
    (25554552, GRAVE),
    (25554552, HAND),
    (67526112, DECK),
    (20071842, DECK),
    (38129297, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(lc), sumtype, tp) && c.is_race_full(duel, RACE_DRAGON, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        g.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_ROKKET], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, 67526112) || c.is_code(duel, 20071842) || c.is_code(duel, 38129297)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, deckthfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, deckthfilter, LOCATION_DECK)
}

fn gythfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_hand(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(gythfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let hg = duel.select_target(tp, gythfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, dg, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, hg, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let (fc, gyc) = tg.split(duel, |c, duel| c.is_location(duel, LOCATION_ONFIELD), None);
    if duel.destroy(fc, REASON_EFFECT)? > 0 {
        duel.send_to_hand(gyc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
