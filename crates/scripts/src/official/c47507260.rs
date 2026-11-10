// Mystic Swordsman LV2
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 47507260;
const MYSTIC_SWORDSMAN_LV4: u32 = 74591968;

pub const LISTED_NAMES: &[u32] = &[MYSTIC_SWORDSMAN_LV4];
// Mystic Swordsman LV4 in the Deck to level up to, and face-down monsters (Tomozaurus) on the opponent's side to attack and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (MYSTIC_SWORDSMAN_LV4, DECK),
    (MYSTIC_SWORDSMAN_LV4, DECK),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_operation(duel, bdop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_condition(duel, spcon::<ID>);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, sptg::<MYSTIC_SWORDSMAN_LV4>);
    e3.set_operation(duel, spop::<MYSTIC_SWORDSMAN_LV4>);
    c.register_effect(duel, e3);
}

pub fn bdop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, CODE, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

pub fn descon(duel: &Duel, a: &Args) -> bool {
    let d = duel.get_attack_target();
    Some(a.e.get_handler(duel)) == duel.get_attacker() && d.is_some_and(|d| d.is_facedown(duel) && d.is_defense_pos(duel))
}

pub fn destg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let d = duel.get_attack_target();
    duel.set_operation_info(0, CATEGORY_DESTROY, d.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, _: &Args) -> Script<()> {
    let d = duel.get_attack_target();
    // The Lua indexes a nil target here and errors.
    unported!(duel, d.is_none());
    if let Some(d) = d
        && d.is_relate_to_battle(duel)
    {
        duel.destroy(d, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn spcon<const CODE: u32>(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, CODE) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, true, true)
}

pub fn sptg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(ft > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, CODE),
                tp,
                LOCATION_HAND | LOCATION_DECK,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

pub fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, CODE),
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
