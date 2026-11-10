// Horus the Black Flame Dragon LV6
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 11224103;
const LV8: u32 = 48229808;

pub const LISTED_NAMES: &[u32] = &[LV8];

// Level 8 in the Deck to Special Summon, and weak Sangans of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (LV8, DECK),
    (LV8, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_condition(duel, aux::bdcon);
    e1.set_operation(duel, bdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(|duel, _, te| te.is_spell_effect(duel)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn bdop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, LV8) && c.is_can_be_special_summoned(duel, e, 0, tp, true, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(
            ft > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
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
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, true, true, POS_FACEUP)? > 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}
