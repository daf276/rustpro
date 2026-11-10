// Mausoleum of White
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 24382602;
const BURST_STREAM_OF_DESTRUCTION: u32 = 17655904;

pub const LISTED_NAMES: &[u32] = &[BURST_STREAM_OF_DESTRUCTION];

// In the hand, with Blue-Eyes White Dragon on the field and in the Deck as the Normal Monster to send and the target, a copy in
// the Graveyard to banish, and Burst Stream of Destruction in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (BURST_STREAM_OF_DESTRUCTION, DECK),
    (BURST_STREAM_OF_DESTRUCTION, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_level(duel, &[1]) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_type(duel, TYPE_TUNER)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, 100);
    duel.set_operation_info(0, CATEGORY_DEFCHANGE, g, 1, tp, 100);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, sc.get_level(duel) as i32 * 100);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, BURST_STREAM_OF_DESTRUCTION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
