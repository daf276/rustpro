// Summoner Monk
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 423585;

// In hand to Summon, with Spells to discard (Polymerization) and Level 4 monsters (Feral Imp) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (41392891, DECK),
    (41392891, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, potg);
    e1.set_operation(duel, poop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_UNRELEASABLE_NONSUM);
    c.register_effect(duel, e5);
}

fn potg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_attack_pos(duel));
    }
    duel.set_operation_info(0, CATEGORY_POSITION, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn poop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_attack_pos(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_discardable(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let handler = Some(a.e.get_handler(duel).into());
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, handler));
    }
    duel.discard_hand(tp, costfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.get_level(duel) == 4 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
