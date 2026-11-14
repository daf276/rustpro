// Left-Hand Shark
use super::c11845050::{efcon, efop, spop, sptg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 47840168;

pub const LISTED_NAMES: &[u32] = &[11845050];

// Copies in hand and the Graveyard with Right-Hand Shark on the field, and Great White to Xyz Summon Number 101: Silent Honor ARK with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (11845050, MZONE),
    (11845050, MZONE),
    (13429800, MZONE),
    (48739166, EXTRA),
    (48739166, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::MaterialFrom(MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_GRAVE));
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_EVENT_PLAYER);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, efcon);
    e3.set_operation(duel, |duel, a| efop(duel, a, EFFECT_INDESTRUCTABLE_EFFECT, 3001));
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, 11845050)
}

// The Lua's spcon(e,c) gets the player as `c`, so its nil check never passes.
fn spcon(duel: &Duel, a: &Args) -> bool {
    let controler = a.e.get_handler(duel).get_controler(duel);
    duel.get_location_count(controler, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(spfilter, controler, LOCATION_ONFIELD, 0, 1, None)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && !a.e.get_handler(duel).is_level(duel, &[4]))
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 4);
        c.register_effect(duel, e1);
    }
    Ok(())
}
