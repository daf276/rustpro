// Genex Spare
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 16828633;

// On the field with Genex Controllers, the other Genex it needs.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (68505803, MZONE), (68505803, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_GENEX]),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        Some(c.into()),
    )
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, 68505803);
    c.register_effect(duel, e1);
    Ok(())
}
