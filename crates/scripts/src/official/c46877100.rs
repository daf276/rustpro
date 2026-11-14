// Scareclaw Acro
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 46877100;

pub const LISTED_SERIES: &[u16] = &[SET_SCARECLAW];
// In the hand, to Special Summon next to a copy on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_value(duel, Value::SpSummonProcedure(hspval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_EMZONE, 0);
    e2.set_target_card(duel, |duel, _, c| sclawfilter(c, duel));
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
}

pub fn sclawfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SCARECLAW])
}

pub fn hspval(duel: &Duel, _: EffectId, c: CardId) -> (i32, i32) {
    let tp = c.get_controler(duel);
    let mut zone = 0;
    for tc in duel.get_matching_group(sclawfilter, tp, LOCATION_MZONE, 0, None).iter() {
        let left_right = u8::from(tc.is_in_main_mzone(duel, None));
        zone |= tc.get_column_zone_full(duel, LOCATION_MZONE, left_right, left_right, tp);
    }
    (0, (zone & ZONES_MMZ) as i32)
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_defense_pos, e.get_handler_player(duel), LOCATION_MZONE, 0, None) as i32 * 300
}
