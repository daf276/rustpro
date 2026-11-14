// Infernoble Knight - Renaud
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 56824871;

pub const LISTED_NAMES: &[u32] = &[];

// In hand with Elemental HERO Burstinatrix (a FIRE Warrior) on the field to summon it, and Burstinatrix and Infernoble Arms -
// Durendal in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (58932615, MZONE),
    (58932615, MZONE),
    (58932615, GRAVE),
    (37478723, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_WARRIOR)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ADD_TYPE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_value(duel, TYPE_TUNER as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_WARRIOR) && !c.is_code(duel, ID)) || c.is_type(duel, TYPE_EQUIP))
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_GRAVE | LOCATION_REMOVED;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, location) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, location, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
