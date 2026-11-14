// Matador Archfiend
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 7622360;
const RITUAL: u32 = 70105073;

pub const LISTED_NAMES: &[u32] = &[RITUAL, ID];
pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];

// Ritual of the Matador with Level 4+ Tributes in hand, Archfiend Soldiers to search, and opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (RITUAL, HAND),
    (49881766, HAND),
    (49881766, HAND),
    (49881766, DECK),
    (49881766, DECK),
    (49881766, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e1.set_target(duel, super::c11248645::thtg::<ID>);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_BATTLED);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    super::c11248645::search_two::<ID>(duel, a.tp)?;
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| bc.is_relate_to_battle(duel)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(CardSet::from), 1, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 1000);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    if let Some(bc) = bc
        && bc.is_relate_to_battle(duel)
        && duel.destroy(bc, REASON_EFFECT)? > 0
    {
        duel.damage(1 - a.tp, 1000, REASON_EFFECT)?;
    }
    Ok(())
}
