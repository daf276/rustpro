// Staysailor Romarin
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 49964567;

// On the field with Plant Normal Monsters (Dark Plant, Queen of Autumn Leaves) to send, Dark Hole in the opponent's hand to destroy
// it and Dark Plant in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13193642, MZONE),
    (4179849, MZONE),
    (13193642, DECK),
    (13193642, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e2.set_target(duel, grtg);
    e2.set_operation(duel, grop);
    c.register_effect(duel, e2);
}

fn trgtfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && duel.is_existing_matching_card(tgfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PLANT) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| trgtfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| trgtfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(tc.into()))?;
        if let Some(gc) = g.get_first()
            && duel.send_to_grave(g, REASON_EFFECT)? > 0
            && gc.is_location(duel, LOCATION_GRAVE)
        {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_count_limit(duel, 1, 0);
            e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & (REASON_BATTLE + REASON_EFFECT) != 0));
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect_full(duel, e1, true);
        }
    }
    Ok(())
}

fn grfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 5) && c.has_level(duel) && c.is_race(duel, RACE_PLANT) && c.is_able_to_grave(duel)
}

fn grtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, grfilter, LOCATION_DECK | LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn grop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, grfilter, LOCATION_DECK | LOCATION_EXTRA)
}
