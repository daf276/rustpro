// DoomZ Break - Diactorus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 31010081;

pub const LISTED_SERIES: &[u16] = &[SET_DOOMZ];
pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE];
// In the Extra Deck, Xyz Summoned with DoomZ VII Seven - Elara and DoomZ V Five - Amalthe, with DoomZ monsters in the hand to
// destroy, Axe of Despair in the Deck to search or send, and Dark Hole to destroy it with them attached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (54265980, MZONE),
    (28877382, MZONE),
    (54265980, HAND),
    (53129443, HAND),
    (40619825, DECK),
    (40619825, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thtgcon);
    e2.set_target(duel, thtgtg);
    e2.set_operation(duel, thtgop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 0, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.destroy(g, REASON_EFFECT)? > 0
        && duel.get_field_group_count(tp, LOCATION_MZONE, LOCATION_MZONE) > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if !dg.is_empty() {
            duel.hint_selection(dg);
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn thtgconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_DOOMZ]) && c.is_monster(duel)) || c.is_code(duel, CARD_MEDIUS_THE_PURE)
}

fn thtgcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.get_overlay_group(duel).is_exists(duel, thtgconfilter, 1, None)
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_spell(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        let send = |duel: &mut Duel| duel.send_to_grave(sc, REASON_EFFECT).map(|_| ());
        aux::to_hand_or_else(duel, sc, tp, |c, duel| c.is_able_to_grave(duel), send, 574)?;
    }
    Ok(())
}
