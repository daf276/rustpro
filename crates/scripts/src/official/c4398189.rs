// Witch of the White Forest
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 4398189;

// In the hand to activate and one in a random zone, with Astellar of the White Forest on the field to protect and make a Tuner, and
// Silvy of the White Forest in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, 0), (25592142, MZONE), (98385955, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_WHITE_FOREST]));
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE == REASON_BATTLE));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, tunertg);
    e3.set_operation(duel, tunerop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WHITE_FOREST]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sg = g.select(duel, tp, 1, 1, None)?;
    if duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? > 0 && sg.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND)) {
        duel.confirm_cards(1 - tp, sg)?;
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            c.is_location(duel, LOCATION_EXTRA) && c.is_attribute(duel, ATTRIBUTE_DARK)
        });
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| c.is_original_attribute(duel, ATTRIBUTE_DARK));
    }
    Ok(())
}

fn nontunertgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WHITE_FOREST]) && !c.is_type(duel, TYPE_TUNER) && c.is_faceup(duel)
}

fn tunertg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && nontunertgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(nontunertgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, nontunertgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn tunerop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
