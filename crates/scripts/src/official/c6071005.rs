// Witchcrafter Marka
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6071005;

// In the hand and the Graveyard, with Witchcrafter monsters on the field for the opponent's Raigeki to destroy, Sangan of the opponent
// to Tribute and Walpurgis in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (64756282, MZONE),
    (21744288, MZONE),
    (32353566, DECK),
    (32353566, DECK),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE + CATEGORY_RELEASE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && (c.is_field_spell(duel) || c.is_continuous_spell(duel)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8, opp: u8) -> bool {
    c.is_previous_set_card(duel, &[SET_WITCHCRAFTER])
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.is_reason_player(duel, opp)
        && c.is_reason(duel, REASON_EFFECT)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| !eg.contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| spconfilter(c, duel, tp, 1 - tp), 1, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(CardId::is_releasable_by_effect, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect(duel, e1);
    }
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_matching_card(tp, CardId::is_releasable_by_effect, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.release(g, REASON_EFFECT)?;
    }
    Ok(())
}
