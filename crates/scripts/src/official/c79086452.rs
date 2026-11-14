// Gimmick Puppet Bisque Doll
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 79086452;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// In the hand with Scissor Arms to discard, and in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (29216967, HAND), (29216967, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, a| !duel.has_flag_effect(a.tp, ID));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_monster(duel) && c.is_discardable(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND, 0, 1, Some(c.into()))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(spcostfilter, tp, LOCATION_HAND, 0, a.c.map(CardSet::from));
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_DISCARD, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_DISCARD | REASON_COST)?;
    Ok(())
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GIMMICK_PUPPET]));
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}
