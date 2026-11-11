// Hexa Spirit of the Ice Barrier
use crate::prelude::*;
use crate::testing::{DECK, MZONE, OPPONENT};

const ID: u32 = 70703416;

// On the field with Mirror Mage of the Ice Barrier, and more in the Deck to send, with a monster on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (9396662, MZONE),
    (9396662, DECK),
    (9396662, DECK),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, -500);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ICE_BARRIER]) && c.is_monster(duel)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        atkfilter,
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 3) && c.is_set_card(duel, &[SET_ICE_BARRIER]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        let c = a.e.get_handler(duel);
        if !(c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel)) {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
