// Gilasaurus
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 45894482;

// In the hand, with Normal Monsters (Petit Dragon) in the opponent's Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (75356564, GRAVE | OPPONENT), (75356564, GRAVE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, opp) = (a.e, 1 - a.tp);
    let filter = |c: CardId, duel: &Duel| c.is_can_be_special_summoned(duel, e, 0, opp, false, false);
    let g = duel.get_matching_group(aux::necro_valley_filter(filter), opp, LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.get_location_count(opp, LOCATION_MZONE) > 0 && duel.select_yes_no(opp, aux::stringid(ID, 1))? {
        duel.hint(HINT_SELECTMSG, opp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, opp, 1, 1, None)?;
        duel.special_summon(sg, 0, opp, opp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
