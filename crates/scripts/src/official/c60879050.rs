// Train Connection
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE};

const ID: u32 = 60879050;

// In hand, with two Level 10 Machines (Night Train Blue Traveler) in the Graveyard to banish and an EARTH Machine (Overdrive) to equip.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (81101309, GRAVE), (81101309, GRAVE), (2311603, MZONE)];

struct Procedure;

impl EquipProcedure for Procedure {
    const EQLIMIT: Option<fn(&Duel, EffectId, CardId) -> i32> = Some(eqlimit);
    const COST: Option<engine::CostFn> = Some(cost);

    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        filter(c, duel)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_base_attack(duel) * 2));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, e, c| e.get_handler(duel).get_equip_target(duel) != Some(c));
    c.register_effect(duel, e3);
}

fn eqlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    (e.get_handler(duel).get_equip_target(duel) == Some(c) && c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_EARTH)).into()
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_EARTH)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE)
        && c.is_level_above(duel, 10)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        duel.is_existing_target(filter, 0, LOCATION_MZONE, LOCATION_MZONE, 1, Some(sg)),
        false,
    )
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let rg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_REMOVE)?;
    duel.remove(rg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}
