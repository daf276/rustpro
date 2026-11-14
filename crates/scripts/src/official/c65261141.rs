// Nightmare Pain
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 65261141;

pub const LISTED_NAMES: &[u32] = &[CARD_YUBEL, ID];
pub const LISTED_SERIES: &[u16] = &[SET_YUBEL];

// Yubel on the field to destroy, more in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (CARD_YUBEL, MZONE),
    (CARD_YUBEL, DECK),
    (CARD_YUBEL, DECK),
    (90829280, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_MUST_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_YUBEL]),
            a.e.get_handler_player(duel),
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e3.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_set_card(duel, &[SET_YUBEL]))));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_YUBEL]));
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_YUBEL) || c.lists_code(duel, CARD_YUBEL)) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 {
        stock::search(duel, a, thfilter, LOCATION_DECK)?;
    }
    Ok(())
}
