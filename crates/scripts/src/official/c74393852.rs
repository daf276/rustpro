// Zoodiac Boarbow
use engine::constants::MSG_ATTACK;

use super::c11510448::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 74393852;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned with five Level 4 Normal Monsters, and the opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (2906250, MZONE),
    (3134241, MZONE),
    (5053103, MZONE),
    (26202165, MZONE | OPPONENT),
];
// The send-to-Graveyard effect needs 12 Xyz Materials, which no Xyz Summon of the card reaches.
pub const CHECKS: &[Check] = &[
    Check::Message(MSG_ATTACK),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<ID, 5, 5>>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e1b.set_value(duel, Value::Card(defval));
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_POSITION);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_condition(duel, condition);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e.get_handler(duel).get_overlay_group(duel);
    g.filter(
        duel,
        |c, duel| c.is_set_card(duel, &[SET_ZOODIAC]) && c.has_non_zero_attack(duel),
        None,
    )
    .get_sum(duel, |c, duel| c.get_attack(duel).into()) as i32
}

fn defval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e.get_handler(duel).get_overlay_group(duel);
    g.filter(
        duel,
        |c, duel| c.is_set_card(duel, &[SET_ZOODIAC]) && c.has_non_zero_defense(duel),
        None,
    )
    .get_sum(duel, |c, duel| c.get_defense(duel).into()) as i32
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && a.e.get_handler(duel).get_overlay_count(duel) >= 12
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND | LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, 0, LOCATION_HAND | LOCATION_ONFIELD);
    if !g.is_empty() && duel.send_to_grave(g, REASON_EFFECT)? > 0 {
        let og = duel.get_operated_group();
        if og.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) == 0 {
            return Ok(());
        }
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
            duel.break_effect()?;
            duel.change_position(c, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}
