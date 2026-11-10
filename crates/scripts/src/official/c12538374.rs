// Treeborn Frog
use crate::prelude::*;
use crate::testing::{Check, GRAVE};

const ID: u32 = 12538374;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the GY, to come back on a Standby Phase while no Spell or Trap is on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) || (c.is_code(duel, ID) && c.is_faceup(duel))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.tp == duel.get_turn_player() && !duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && !duel.is_existing_matching_card(filter, tp, LOCATION_ONFIELD, 0, 1, None) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
