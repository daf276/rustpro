// Pressured Planet Wraitsoth
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 71832012;

pub const LISTED_SERIES: &[u16] = &[SET_KASHTIRA];
pub const LISTED_NAMES: &[u32] = &[73542331];

// In the hand, with Kashtira monsters in the Deck to search and Kashtira Shangri-Ira on the field to chain its effects to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (73542331, MZONE),
    (73542331, MZONE),
    (32909498, DECK),
    (68304193, DECK),
    (32909498, MZONE),
    (68304193, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(duel, Value::Card(atkdefval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, descon);
    e4.set_target(duel, destg);
    e4.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_KASHTIRA]) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn atkdefval(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    let g = duel.get_matching_group(CardId::is_faceup, 0, LOCATION_MZONE, LOCATION_MZONE, None);
    100 * g.get_bin_class_count(duel, |c, duel| c.get_attribute(duel).into()) as i32
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CONTROLER);
    let code1: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CODE);
    let code2: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CODE2);
    p == a.tp && (code1 == 73542331 || code2 == 73542331)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
