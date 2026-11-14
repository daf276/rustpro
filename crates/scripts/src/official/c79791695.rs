// CXyz Chaos Barian Hope Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 79791695;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER_C];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

// In the Extra Deck, Xyz Summoned with three Level 7 Normal Monsters or Number C104 (Number C104 on the field, with materials to detach).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (49456901, MZONE),
    (49456901, MZONE),
    (36996508, MZONE),
    (36996508, MZONE),
    (36996508, MZONE),
    (36996508, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 1000));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_NUMBER_C]), 1, None)
    });
    e2.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_DECK));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, attachtg);
    e3.set_operation(duel, attachop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    if c.is_facedown(duel) || !c.is_set_card_full(duel, &[SET_NUMBER_C], Some(lc), SUMMON_TYPE_XYZ as u64, tp) {
        return false;
    }
    crate::official::xyz_number(c.get_code(duel)).is_some_and(|no| (101..=107).contains(&no))
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_xyz_monster(duel) && duel.get_field_group_count(1 - a.tp, LOCATION_DECK, 0) >= 2);
    }
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_decktop_group(1 - tp, 2);
    if c.is_relate_to_effect(duel, e) && g.len() == 2 {
        duel.disable_shuffle_check(true);
        duel.overlay(c, g)?;
    }
    Ok(())
}
