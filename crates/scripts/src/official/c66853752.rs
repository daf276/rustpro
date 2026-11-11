// Fishborg Launcher
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 66853752;

// In the Graveyard with only WATER monsters (Slime Toad, Fishborg Harpooner) there too.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, HAND), (68638985, GRAVE), (68638985, MZONE), (57420265, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_attribute(duel, ATTRIBUTE_WATER))));
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.get_code(duel) != ID && c.is_monster(duel)
}

fn only_water_in_grave(duel: &Duel, tp: u8) -> bool {
    let g = duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None);
    !g.is_empty() && g.filter_count(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_WATER), None) == g.len()
}

fn condition(duel: &Duel, a: &Args) -> bool {
    only_water_in_grave(duel, a.tp)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && only_water_in_grave(duel, tp)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
