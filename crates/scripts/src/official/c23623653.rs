// Spiritual Beast Tamer Lara
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 23623653;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];
pub const LISTED_NAMES: &[u32] = &[ID];
// In hand to discard with Ritual Beast monsters to Normal Summon, in the Graveyard to replace the destruction of Ritual Beast
// monsters by the opponent's Raigeki, and Ritual Beast monsters in the Deck to summon once banished.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (14513016, HAND),
    (40907115, HAND),
    (88123329, MZONE),
    (88123329, MZONE),
    (88123329, DECK),
    (14513016, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Replaced,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, reptg);
    e2.set_operation(duel, |duel, a| {
        duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT | REASON_REPLACE)?;
        Ok(())
    });
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_REMOVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(nsfilter, a.tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.summon(tp, tc, true, None)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_RITUAL_BEAST])
        && c.is_on_field(duel)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_status(duel, STATUS_DESTROY_CONFIRMED)
            && c.is_able_to_remove(duel)
            && a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK, POS_FACEUP)
}
