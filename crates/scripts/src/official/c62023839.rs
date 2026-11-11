// Fishborg Archer
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 62023839;

// In the Graveyard with an empty field and a WATER monster (Slime Toad) to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (68638985, HAND), (68638985, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_discardable(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, desop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
