// Mermail Abyssgaios
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 74371660;

// In the Extra Deck, Xyz Summoned with two Mermail Abyssteus, a Level 7 WATER monster, with a second copy in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (22446869, MZONE), (22446869, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, sc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(sc), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_level_above(duel, 5));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) != 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_negatable_monster(duel) && c.get_attack(duel) < atk
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let atk = a.e.get_handler(duel).get_attack(duel);
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, atk), a.tp, 0, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let atk = c.get_attack(duel);
    let g = duel.get_matching_group(|c, duel| filter(c, duel, atk), a.tp, 0, LOCATION_MZONE, None);
    for tc in g {
        super::c97268402::negate(duel, c, tc, 0);
        duel.negate_related_chain(tc, RESET_TURN_SET);
    }
    Ok(())
}
