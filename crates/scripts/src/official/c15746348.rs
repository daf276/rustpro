// Eria the Water Channeler
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 15746348;

pub const LISTED_SERIES: &[u16] = &[SET_CHARMER];
// In the hand with Great White (Level 4 WATER) to discard and Kairyu-Shin (Level 5 WATER) in the Deck, and Great White on the
// field to be destroyed by the opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (13429800, HAND),
    (76634149, DECK),
    (76634149, DECK),
    (13429800, MZONE),
    (13429800, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let tp = a.tp;
        let spconfilter =
            |c: CardId, duel: &Duel| c.is_previous_controler(duel, tp) && c.is_previous_attribute_on_field(duel, ATTRIBUTE_WATER);
        a.eg.unwrap_or_default().is_exists(duel, spconfilter, 1, None)
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn thcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let lv = c.get_level(duel);
    c.is_attribute(duel, ATTRIBUTE_WATER)
        && c.is_discardable(duel)
        && duel.is_existing_matching_card(|c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_level_above(duel, lv) && c.is_able_to_hand(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let filter = |cc, duel: &Duel| thcostfilter(cc, duel, tp);
    if chk == 0 {
        return Ok(c.is_discardable(duel) && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    let lv = g.get_first().map_or(0, |tc| tc.get_level(duel));
    a.e.set_label(duel, i64::from(lv));
    g.insert(c);
    duel.send_to_grave(g, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let lv = a.e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && re.get_handler(duel).is_attribute_except(duel, ATTRIBUTE_WATER)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
