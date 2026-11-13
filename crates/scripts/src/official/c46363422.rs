// Skilled White Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 46363422;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_SPELL];

pub const LISTED_NAMES: &[u32] = &[CARD_BUSTER_BLADER];

// On the field, with Spellbook Organization in the hand to activate for Spell Counters and Buster Blader in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (96677818, HAND),
    (96677818, HAND),
    (96677818, HAND),
    (96677818, HAND),
    (CARD_BUSTER_BLADER, DECK),
    (CARD_BUSTER_BLADER, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_SPELL);
    c.set_counter_limit(duel, COUNTER_SPELL, 3);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_CHAINING);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, acop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

pub fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.re
        .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_spell_effect(duel))
        && c.get_flag_effect(duel, 1) > 0
    {
        c.add_counter(duel, COUNTER_SPELL, 1);
    }
    Ok(())
}

pub fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_counter(duel, COUNTER_SPELL) == 3 && c.is_releasable(duel));
    }
    duel.release(c, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_BUSTER_BLADER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    let location = LOCATION_DECK | LOCATION_HAND | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 0, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let location = LOCATION_DECK | LOCATION_HAND | LOCATION_GRAVE;
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| filter(c, duel, e, tp)),
        tp,
        location,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
