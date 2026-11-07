// Micro Coder
use super::c30114823::{extracon, extraval};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 2347477;

// Another on the field and in the hand as Link material for Code Talker, and Leotron to add when it was on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (53413628, EXTRA), (47226949, DECK)];
// The search, which finds only Leotron with the label thcon sets, and this card as extra material from the hand.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EFFECT_EXTRA_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_extra_material_operation(duel, extracon::<ID>);
    e1.set_value(duel, Value::ExtraMaterial(extraval::<ID>));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition_mut(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.e.set_label(duel, 0);
    if c.is_previous_location(duel, LOCATION_ONFIELD) {
        a.e.set_label(duel, 1);
    }
    c.is_location(duel, LOCATION_GRAVE)
        && c.is_previous_location(duel, LOCATION_ONFIELD | LOCATION_HAND)
        && a.r == REASON_LINK
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_set_card(duel, &[SET_CODE_TALKER]))
}

fn thfilter(c: CardId, duel: &Duel, chk: i64) -> bool {
    ((c.is_set_card(duel, &[SET_CYNET]) && c.is_spell_trap(duel)) || (chk == 1 && c.is_race(duel, RACE_CYBERSE) && c.is_level(duel, &[4])))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, label) = (a.tp, a.e.get_label(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, label), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, label) = (a.tp, a.e.get_label(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, label), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
