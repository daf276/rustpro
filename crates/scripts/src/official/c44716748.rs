// Trito Enneacraft - "exapatisIA"
use super::c28454232::{flip_quick, register_counter, register_set_summon, set_filter};
use super::c92171126::{register_reveal, thop, thtg};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 44716748;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone, face-down on the field and in the Deck to reveal and search, with the opponent's Set Dark
// Holes to destroy and copies in the hand, whose face-down summon it answers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
    (ID, HAND | OPPONENT),
    (ID, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_reveal::<ID>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_FLIP);
    e3.set_target(duel, fliptg);
    e3.set_operation(duel, flipop);
    c.register_effect(duel, e3);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_TOHAND + CATEGORY_SEARCH, 0);
    e4.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_facedown(duel) && a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_has_category(duel, CATEGORY_SET))
    });
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    register_set_summon::<ID, 3>(duel, c);
}

fn fliptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - a.tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn flipop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(tp, |c, duel| c.is_spell_trap(duel), tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if dg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(dg);
    if duel.destroy(dg, REASON_EFFECT)? > 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        let e = a.e;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| set_filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}
