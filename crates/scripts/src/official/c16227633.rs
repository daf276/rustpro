// Mementotlan Bone Back
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 16227633;

pub const LISTED_SERIES: &[u16] = &[SET_MEMENTO];
pub const LISTED_NAMES: &[u32] = &[CARD_MEMENTOAL_TECUHTLICA];

// In the hand and the GY, with Mementotlan monsters on the field for the opponent's Raigeki and in the GY for their Called by
// the Grave, and Mementoal Tecuhtlica and Mementotlan monsters in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (18165869, MZONE),
    (55272555, MZONE),
    (54550967, GRAVE),
    (81677154, GRAVE),
    (CARD_MEMENTOAL_TECUHTLICA, DECK),
    (54207171, DECK),
    (17943271, DECK),
    (12580477, HAND | OPPONENT),
    (24224830, HAND | OPPONENT),
    (24224830, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_LEAVE_FIELD);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_LEAVE_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spmanycon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, spmanytg);
    e2.set_operation(duel, spmanyop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_set_card(duel, &[SET_MEMENTO])
        && c.get_reason_player(duel) == 1 - tp
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_MEMENTOAL_TECUHTLICA) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}

fn outgravecfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_MEMENTO]) && c.is_previous_controler(duel, tp)
}

fn spmanycon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| outgravecfilter(c, duel, a.tp), 1, None))
}

fn spmanyfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spmanytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spmanyfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
    )
}

fn spmanyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spmanyfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    let classes = g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    let mut ct = (duel.get_location_count(tp, LOCATION_MZONE) as i64).min(classes as i64);
    if ct <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    let ct = ct as usize;
    let sg = aux::select_unselect_group(duel, g, ct, ct, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
