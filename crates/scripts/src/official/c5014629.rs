// Submersible Carrier Aero Shark
use crate::prelude::*;
use crate::testing::{Check, MZONE, REMOVED};

const ID: u32 = 5014629;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// Burglar (a Normal Monster) banished, which is face-down about half the time.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (6297941, REMOVED), (6297941, REMOVED), (6297941, REMOVED)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_REMOVED, 0, None) as u32;
    duel.set_target_player(1 - tp);
    duel.set_target_param(ct * 100);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ct as i32 * 100);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_REMOVED, 0, None) as u32;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    duel.damage(p, ct * 100, REASON_EFFECT)?;
    Ok(())
}
