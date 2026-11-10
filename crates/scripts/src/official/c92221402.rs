// Chaotic Elements
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92221402;

// Copies in the hand and Graveyard, K9-66a Jokul and K9-66b Lantern (LIGHT/DARK Level 5 Pyro and Aqua) in the Deck to add and
// on the opponent's field, and three Pyro and Aqua monsters in the Graveyard to destroy with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (28642461, DECK),
    (55031170, DECK),
    (28642461, MZONE | OPPONENT),
    (2830619, GRAVE),
    (5464695, GRAVE),
    (4042268, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, ctrlcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, crtltg);
    e2.set_operation(duel, crtlop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
        && c.is_race(duel, RACE_PYRO | RACE_AQUA)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, tc)?;
        let c = e.get_handler(duel);
        let exc = c.is_relate_to_effect(duel, e).then_some(c);
        if duel.is_existing_matching_card(|c, duel| c.is_race(duel, RACE_PYRO | RACE_AQUA), tp, LOCATION_GRAVE, 0, 3, None)
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, exc.map(Into::into))
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, exc.map(Into::into))?;
            if sg.is_empty() {
                return Ok(());
            }
            duel.hint_selection(sg);
            duel.break_effect()?;
            duel.destroy(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn ctrlconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK) && c.is_race(duel, RACE_PYRO | RACE_AQUA) && c.is_faceup(duel)
}

fn ctrlcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(ctrlconfilter, a.tp, 0, LOCATION_MZONE, 1, None)
}

fn crtltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn crtlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
    }
    Ok(())
}
