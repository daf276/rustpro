// Ryzeal Duo Drive
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 7511613;

pub const LISTED_SERIES: &[u16] = &[SET_RYZEAL];
// In the Extra Deck, Xyz Summoned with Ryzeal monsters on the field, with others in the Graveyard to attach and in the Deck
// to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (8633261, MZONE),
    (35844557, MZONE),
    (72238166, MZONE),
    (34022970, GRAVE),
    (34022970, GRAVE),
    (8633261, DECK),
    (35844557, DECK),
    (72238166, DECK),
    (84433129, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| {
            let sign = if c.is_controler(duel, e.get_handler_player(duel)) {
                100
            } else {
                -100
            };
            sign * e.get_handler(duel).get_overlay_count(duel) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && c.is_monster(duel) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(|tc, duel| attachfilter(tc, duel, c, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(tp, |tc, duel| attachfilter(tc, duel, c, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.hint_selection(tc);
        duel.overlay(c, tc)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        return Ok(duel.check_remove_overlay_card(tp, 1, 0, 2, REASON_EFFECT)
            && g.len() >= 2
            && aux::select_unselect_group_check(duel, g, 2, 2, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.remove_overlay_card(tp, 1, 0, 2, 2, REASON_EFFECT)? != 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() < 2 {
        return Ok(());
    }
    let thg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !thg.is_empty() {
        duel.send_to_hand(thg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, thg)?;
    }
    Ok(())
}
