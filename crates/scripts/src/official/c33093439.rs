// Cyber Eltanin
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 33093439;

// In the hand with LIGHT Machine monsters, Cyber Dragons, on the field and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, GRAVE),
    (CARD_CYBER_DRAGON, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_able_to_remove_as_cost(duel)
        && (!c.is_location(duel, LOCATION_MZONE) || c.is_faceup(duel))
}

fn mzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.get_sequence(duel) < 5
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let mg = duel.get_matching_group(cfilter, tp, LOCATION_MZONE, 0, None);
    let gg = match duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION) {
        None => duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None),
        Some(_) => CardSet::default(),
    };
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    (!mg.is_empty() && (ft > 0 || (mg.is_exists(duel, mzfilter, 1, None) && ft > -1))) || (!gg.is_empty() && ft > 0)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let tp = a.tp;
    let location = match duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION) {
        Some(_) => LOCATION_MZONE,
        None => LOCATION_MZONE | LOCATION_GRAVE,
    };
    let g = duel.get_matching_group(cfilter, tp, location, 0, None);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    e1.set_value(duel, g.len() as i32 * 500);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e2);
    Ok(())
}

fn others(duel: &Duel, a: &Args) -> CardSet {
    duel.get_matching_group(
        CardId::is_faceup,
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        Some(a.e.get_handler(duel).into()),
    )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = others(duel, a);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = others(duel, a);
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}
