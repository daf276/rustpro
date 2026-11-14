// Maliss in Underground
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 68337209;

pub const LISTED_SERIES: &[u16] = &[SET_MALISS];
// With Maliss monsters in the Deck to banish, and Maliss <Q> Hearts Crypter for the opponent's attacks to be forced onto.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (20938824, DECK), (96676583, DECK), (21848500, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, atkupcon);
    e2.set_target_card(duel, |duel, _, c| atkfilter(c, duel));
    e2.set_value(duel, 3000);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_condition(duel, atktgcon);
    e3.set_value(duel, Value::Card(|duel, _, c| i32::from(!atkfilter(c, duel))));
    c.register_effect(duel, e3);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(
        0,
        CATEGORY_REMOVE,
        None,
        1,
        a.tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALISS]) && c.is_able_to_remove(duel)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(
        aux::necro_valley_filter(rmfilter),
        tp,
        LOCATION_DECK | LOCATION_HAND | LOCATION_GRAVE,
        0,
        None,
    );
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if !sg.is_empty() {
            duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALISS]) && c.is_type(duel, TYPE_LINK) && c.is_faceup(duel)
}

fn atkupconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALISS]) && c.is_trap(duel) && c.is_faceup(duel)
}

fn atkupcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(atkupconfilter, a.e.get_handler_player(duel), LOCATION_REMOVED, 0, None);
    g.get_class_count(duel, |c, duel| c.get_code(duel).into()) >= 3
}

fn atktgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(atkfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}
