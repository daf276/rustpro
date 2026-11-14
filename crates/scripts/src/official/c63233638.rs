// Megalith Phaleg
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 63233638;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

const MEGALITH_OCH: u32 = 36849933;

struct TargetParams;

impl RitualParams for TargetParams {
    const FORCEDSELECTION: Option<ForcedSelection> = Some(forcedselection);

    fn filter_with_effect(c: CardId, duel: &Duel, e: EffectId) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH]) && c != e.get_handler(duel)
    }
}

struct OperationParams;

impl RitualParams for OperationParams {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH])
    }
}

// Discarded from the hand to Ritual Summon Megalith Och, with a monster on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (MEGALITH_OCH, HAND), (MEGALITH_OCH, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

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
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| {
            300 * duel.get_matching_group_count(CardId::is_ritual_monster, e.get_handler_player(duel), LOCATION_GRAVE, 0, None) as i32
        }),
    );
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn forcedselection(duel: &Duel, e: EffectId, _: u8, g: CardSet, _: CardId) -> (bool, bool) {
    let c = e.get_handler(duel);
    (!g.is_contains(c), g.is_contains(c))
}
