// Malevolent Nuzzler
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 99597615;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip Elemental HERO Burstinatrix (a Level 3 FIRE Warrior Normal Monster), with Dark Hole in the hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (58932615, MZONE), (53129443, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 700);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TODECK);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e4.set_cost(duel, Cost::pay_lp::<500>);
    e4.set_target(duel, tdtg);
    e4.set_operation(duel, tdop);
    c.register_effect(duel, e4);
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    }
    Ok(())
}
