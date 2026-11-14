// Megalith Bethor
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

use super::c25726386::{OperationParams, TargetParams};

const ID: u32 = 99628747;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

// In the hand to Ritual Summon a copy or Megalith Och by Tributing Black Luster Soldier (Level 8), with Ritual Monsters in the GY
// to count.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (36849933, HAND),
    (10000100, HAND),
    (10000100, MZONE),
    (36849933, GRAVE),
    (30208479, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, Ritual::target::<TargetParams, RITPROC_GREATER>);
    e1.set_operation(duel, Ritual::operation::<OperationParams, RITPROC_GREATER>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel));
    }
    let rituals = duel.get_matching_group(CardId::is_ritual_monster, tp, LOCATION_GRAVE, 0, None);
    let ct = rituals.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
