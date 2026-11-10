// Pilgrim Reaper
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 45742626;

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters (Frostosaurus, Boulder Tortoise).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (6631034, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(value));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DECKDES);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn value(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    let dark = |c: CardId, duel: &Duel| c.is_attribute(duel, ATTRIBUTE_DARK);
    duel.get_matching_group_count(dark, 0, LOCATION_GRAVE, LOCATION_GRAVE, None) as i32 * 200
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 5) && duel.is_player_can_discard_deck(1 - tp, 5));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, PLAYER_ALL, 5);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g1 = duel.get_decktop_group(tp, 5);
    let g2 = duel.get_decktop_group(1 - tp, 5);
    g1.merge(g2);
    duel.disable_shuffle_check(true);
    duel.send_to_grave(g1, REASON_EFFECT)?;
    Ok(())
}
