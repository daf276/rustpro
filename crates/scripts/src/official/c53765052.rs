// Diabellze the Original Sinkeeper
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53765052;

// In the hand with a "Sinful Spoils" Spell (Sinful Spoils of Subversion - Snake-Eye) in the Graveyard, and Set Spells and Traps on
// both sides for the destruction to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (24081957, GRAVE),
    (24081957, HAND),
    (24081957, HAND | OPPONENT),
    (24081957, SZONE),
    (24081957, SZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| {
            re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && !re.get_handler(duel).is_location(duel, LOCATION_SZONE)
        }),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_SSET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            |c, duel| c.is_set_card(duel, &[SET_SINFUL_SPOILS]),
            0,
            LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            None,
        )
}

fn desrescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1, false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| {
            desrescon(duel, sg, tp)
        }));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| desrescon(duel, sg, tp), tp, HINTMSG_DESTROY)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), 2, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
