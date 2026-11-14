// Saffira, Queen of Dragons
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 56350972;

pub const LISTED_NAMES: &[u32] = &[80566312];

// Ritual Summoned by Prayers of the Voiceless Voice Tributing Lo, with LIGHT monsters in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (52472775, HAND),
    (52472775, HAND),
    (25801745, MZONE),
    (25801745, MZONE),
    (25801745, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_condition(duel, regcon1);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_condition(duel, regcon2);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn regcon1(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_ritual_summoned(duel)
}

fn regfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_previous_location(duel, LOCATION_HAND | LOCATION_DECK)
}

fn regcon2(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, regfilter, 1, None)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_player_can_draw(tp, 2);
    let b2 = duel.get_field_group_count(1 - tp, LOCATION_HAND, 0) != 0;
    let b3 = duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let opval: Vec<u64> = [(b1, 1), (b2, 2), (b3, 3)]
        .into_iter()
        .filter(|&(b, _)| b)
        .map(|(_, sel)| sel)
        .collect();
    let ops: Vec<u64> = opval.iter().map(|&sel| aux::stringid(ID, sel)).collect();
    let op = duel.select_option(tp, &ops)?;
    let sel = usize::try_from(op).ok().and_then(|op| opval.get(op)).copied().unwrap_or(0);
    e.set_label(duel, sel as i64);
    match sel {
        1 => {
            e.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
            duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
        }
        2 => {
            e.set_category(duel, CATEGORY_HANDES);
            duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 1);
        }
        _ => {
            e.set_category(duel, CATEGORY_TOHAND);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
        }
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            duel.draw(tp, 2, REASON_EFFECT)?;
            duel.shuffle_hand(tp)?;
            duel.break_effect()?;
            duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
        }
        2 => {
            let g = duel.get_field_group(tp, 0, LOCATION_HAND);
            if g.is_empty() {
                return Ok(());
            }
            let sg = g.random_select(duel, tp, 1);
            duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
        }
        _ => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
    }
    Ok(())
}
