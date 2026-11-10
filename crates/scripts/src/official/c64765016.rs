// Paleozoic Marrella
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, SZONE};

const ID: u32 = 64765016;

// A copy in the Graveyard to Special Summon in response to another Set copy activating, with Bottomless Trap Hole in the Deck to
// send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, SZONE), (ID, SZONE), (29401950, DECK), (29401950, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = paleozoic_spsummon::<ID>(duel, c);
    e2.set_description(duel, aux::stringid(ID, 0));
    c.register_effect(duel, e2);
}

// The second effect all Paleozoic Traps share, left unregistered: Special Summon itself from the Graveyard as a monster.
pub fn paleozoic_spsummon<const CODE: u32>(duel: &mut Duel, c: CardId) -> EffectId {
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg::<CODE>);
    e2.set_operation(duel, spop::<CODE>);
    e2
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_trap_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn can_summon<const CODE: u32>(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(SET_PALEOZOIC, TYPE_MONSTER | TYPE_NORMAL, 1200, 0, 2, RACE_AQUA, ATTRIBUTE_WATER);
    duel.is_player_can_special_summon_monster(tp, CODE, stats)
}

fn sptg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, CODE) == 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon::<CODE>(duel, tp));
    }
    c.register_flag_effect(duel, CODE, RESET_CHAIN, 0, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if c.is_relate_to_effect(duel, a.e) && can_summon::<CODE>(duel, tp) {
        c.add_monster_attribute(duel, TYPE_NORMAL);
        c.assume_property(duel, ASSUME_RACE, RACE_AQUA);
        duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
        c.add_monster_attribute_complete(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3101);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, Value::Effect(|duel, _, re| re.is_monster_effect(duel)));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 3300);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e2.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e2, true);
        duel.special_summon_complete()?;
    }
    Ok(())
}
