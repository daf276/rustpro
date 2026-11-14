// Gimmick Puppet Rouge Doll
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 63825486;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// In the hand with Number 40 (Rank 8) in the Extra Deck to reveal and Dreary Doll (Level 8) in the Deck to summon, and on the field
// for the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (75433814, EXTRA),
    (75433814, EXTRA),
    (92418590, DECK),
    (92418590, DECK),
    (92418590, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    register_self_to_hand::<ID>(duel, c);
}

// The "add this card to your hand" trigger and the activity counter Fiendish Knight shares, with its own strings.
pub fn register_self_to_hand<const CODE: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(CODE, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (CODE, 1));
    e2.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND));
    e2.set_cost(duel, thcost::<CODE>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        CODE,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_set_card(duel, &[SET_GIMMICK_PUPPET])),
    );
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let rank = c.get_rank(duel);
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET])
        && c.is_type(duel, TYPE_XYZ)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|dc, duel| spfilter(dc, duel, e, tp, rank), tp, LOCATION_DECK, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rank: u32) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_level(duel, &[rank]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(rc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(false);
    };
    let rank = rc.get_rank(duel);
    e.set_label(duel, i64::from(rank));
    duel.confirm_cards(1 - tp, rc)?;
    duel.shuffle_extra(tp)?;
    register_extra_lock::<ID>(duel, a);
    Ok(false)
}

fn register_extra_lock<const CODE: u32>(duel: &mut Duel, a: &Args) {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(CODE, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_GIMMICK_PUPPET])
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        Some(c.into()),
        2,
        tp,
        (LOCATION_HAND | LOCATION_DECK) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return Ok(());
    }
    let rank = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g = duel.select_matching_card(tp, |dc, duel| spfilter(dc, duel, e, tp, rank), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        g.insert(c);
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

pub fn thcost<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(CODE, a.tp, ACTIVITY_SPSUMMON) == 0);
    }
    register_extra_lock::<CODE>(duel, a);
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
