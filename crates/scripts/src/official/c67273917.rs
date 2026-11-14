// Igknight Cavalier
use crate::prelude::*;
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 67273917;

// In the Pendulum Zone with another Igknight (Igknight Crusader), and Igknight Gallant (a FIRE Warrior) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (61639289, PZONE), (96802306, DECK), (96802306, DECK)];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_set_card(duel, &[SET_IGKNIGHT]),
        a.tp,
        LOCATION_PZONE,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    let g = duel.get_field_group(tp, LOCATION_PZONE, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let dg = duel.get_field_group(tp, LOCATION_PZONE, 0);
    if dg.len() < 2 || duel.destroy(dg, REASON_EFFECT)? != 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(filter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
