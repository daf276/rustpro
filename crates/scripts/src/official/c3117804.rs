// Superheavy Samurai Big Benkei
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 3117804;

pub const LISTED_SERIES: &[u16] = &[SET_SUPERHEAVY_SAMURAI];
// Copies in the hand to Tribute Summon over Superheavy Samurai Trumpeter and Motorbike on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (64373401, MZONE), (83334932, MZONE), (83334932, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI]));
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.change_position_full(c, [POS_FACEUP_DEFENSE, 0, POS_FACEUP_ATTACK, 0], false)?;
    }
    Ok(())
}
