// Battlewasp - Pin the Bullseye
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 65899613;

// A copy on the field and one in the hand to Special Summon, with Battlewasp - Sting the Poison on the field as the Insect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (28388927, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_INSECT)),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, ID)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(damfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_target_player(1 - tp);
    let dam = duel.get_matching_group_count(damfilter, tp, LOCATION_MZONE, 0, None) as u32 * 200;
    duel.set_target_param(dam);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = duel.get_matching_group_count(damfilter, tp, LOCATION_MZONE, 0, None) as u32 * 200;
    duel.damage(p, dam, REASON_EFFECT)?;
    Ok(())
}
