// Raidraptor - Final Fortress Falcon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 43047672;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 3;
}

// On the field with monsters of the opponent attached by Xyz Import, Key Mace to battle and Rise Falcon in the Graveyard and banished.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (73887236, GRAVE),
    (73887236, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, imcon);
    e1.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_count_limit(duel, 2, 0);
    e3.set_condition(duel, atcon);
    e3.set_cost(duel, atcost);
    e3.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn imfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RAIDRAPTOR]) && c.is_type(duel, TYPE_XYZ)
}

fn imcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_overlay_group(duel).is_exists(duel, imfilter, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_RAIDRAPTOR]) && c.is_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_REMOVED, 0, None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_REMOVED, 0, None);
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT | REASON_RETURN)?;
    }
    Ok(())
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) && aux::bdocon(duel, a) && duel.check_mut(|duel| c.can_chain_attack_full(duel, 0, false))
}

fn atfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RAIDRAPTOR])
        && c.is_type(duel, TYPE_XYZ)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn atcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(atfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, atfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}
