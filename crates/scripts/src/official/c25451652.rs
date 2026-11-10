// Darklord Morningstar
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 25451652;

// In the hand to be Tribute Summoned with Prevent Rats, with Darklords (Gulgolet, Ixchel) in the Deck to Special Summon and
// Elfnote Lucina in the opponent's monster zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (84031359, DECK),
    (84031359, DECK),
    (52840267, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(2),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_tribute_summoned(duel));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    e3.set_condition(duel, |duel, a| {
        let e = a.e;
        duel.is_existing_matching_card(
            cfilter,
            e.get_handler_player(duel),
            LOCATION_MZONE,
            0,
            1,
            Some(e.get_handler(duel).into()),
        )
    });
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DECKDES + CATEGORY_RECOVER);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(ctfilter, tp, 0, LOCATION_MZONE, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g1 = duel.get_matching_group(ctfilter, tp, 0, LOCATION_MZONE, None);
    let g2 = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    let limit = if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        5
    };
    let ct = limit.min(g1.len()).min(duel.get_location_count(tp, LOCATION_MZONE).max(0) as usize);
    if ct > 0 && !g2.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let tg = g2.select(duel, tp, 1, ct as u8, None)?;
        duel.special_summon(tg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARKLORD])
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, ct));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, ct as i32);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, ct as i32 * 1000);
    Ok(false)
}

fn ctfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_GRAVE) && c.is_set_card(duel, &[SET_DARKLORD])
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct1 = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if ct1 > 0 && duel.discard_deck(tp, ct1 as u16, REASON_EFFECT)? != 0 {
        let ct2 = duel.get_operated_group().filter_count(duel, ctfilter2, None);
        if ct2 > 0 {
            duel.recover(tp, ct2 as u32 * 500, REASON_EFFECT)?;
        }
    }
    Ok(())
}
