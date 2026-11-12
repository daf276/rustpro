// Number 17: Leviathan Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 69610924;
pub const XYZ_NUMBER: u32 = 17;

// In the Extra Deck, Xyz Summoned with two Level 3 monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (3573512, MZONE),
    (4931562, MZONE),
    (6103114, MZONE),
    (1761063, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    c.register_effect(duel, e2);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
