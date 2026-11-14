// Heraldic Beast Gryphon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 60268386;

pub const LISTED_SERIES: &[u16] = &[SET_HERALDIC_BEAST];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand with Heraldic Beast Eale in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (82315772, DECK), (82315772, DECK), (82315772, DECK)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DOUBLE_XYZ_MATERIAL);
    e2.set_value(duel, 1);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_xyz_material_operation(duel, |duel, _, xyz, _, minxyzct| {
        minxyzct >= 3 && xyz.is_set_card(duel, &[SET_NUMBER])
    });
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERALDIC_BEAST]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel) && !c.is_code(duel, ID)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    super::c97662494::register_xyz_restrictions(duel, c, tp, aux::stringid(ID, 1));
    Ok(())
}
