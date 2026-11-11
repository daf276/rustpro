// Dragunity Couse
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 29253591;

// Dragunity Falchion in hand to equip a copy from the Deck to one on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, DECK), (ID, DECK), (8039562, HAND), (8039562, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_set_card(duel, &[SET_DRAGUNITY]))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_SYNCHRO_LEVEL);
    e2.set_value(duel, Value::Card(slevel));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    e3.set_target(duel, stock::special_summon_self_target);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn slevel(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    4 << 16 | e.get_handler(duel).get_level(duel) as i32
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    Ok(())
}
