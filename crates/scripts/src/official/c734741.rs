// Rubic, Malebranche of the Burning Abyss
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 734741;

// Copies in the hand to Special Summon onto an empty field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_SELF_DESTROY);
    e1.set_condition(duel, sdcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, sscon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, ssop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_value(duel, Value::Card(synlimit));
    c.register_effect(duel, e3);
}

fn synlimit(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(!c.is_set_card(duel, &[SET_BURNING_ABYSS]))
}

fn sdfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_faceup(duel) || !c.is_set_card(duel, &[SET_BURNING_ABYSS])
}

fn sdcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(sdfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn sscon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(CardId::is_spell_trap, a.tp, LOCATION_ONFIELD, 0, 1, None)
}

fn ssop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
