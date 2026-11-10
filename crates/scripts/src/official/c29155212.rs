// Pumpking the King of Ghosts
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 29155212;
const CASTLE_OF_DARK_ILLUSIONS: u32 = 62121;

pub const LISTED_NAMES: &[u32] = &[CASTLE_OF_DARK_ILLUSIONS];

// On the field with face-up Castle of Dark Illusions, which are flipped by the opponent's Sangan attacking.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (CASTLE_OF_DARK_ILLUSIONS, MZONE),
    (CASTLE_OF_DARK_ILLUSIONS, MZONE),
    (CASTLE_OF_DARK_ILLUSIONS, MZONE),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(adval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn castle_on_field(duel: &Duel) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CASTLE_OF_DARK_ILLUSIONS));
    duel.is_existing_matching_card(filter, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
}

fn adval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let bonus = if castle_on_field(duel) { 100 } else { 0 };
    bonus + c.get_flag_effect(duel, ID) as i32 * 100
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_flag_effect(duel, ID) <= 4 && castle_on_field(duel)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 1);
    }
    Ok(())
}
