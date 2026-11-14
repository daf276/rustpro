// "Infernoble Arms - Almace"
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 98319530;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOBLE_ARMS];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip Elemental HERO Burstinatrix (a FIRE Warrior), with Infernoble Arms - Durendal in the Deck to equip, a
// Burstinatrix in the GY to add, and Dark Hole to destroy the equipped monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (58932615, MZONE),
    (58932615, MZONE),
    (58932615, GRAVE),
    (37478723, DECK),
    (37478723, DECK),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn eqtcfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_faceup(duel) && ec.check_equip_target(duel, c)
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_equip_spell(duel)
        && c.is_set_card(duel, &[SET_INFERNOBLE_ARMS])
        && !c.is_code(duel, ID)
        && c.check_unique_on_field(duel, tp)
        && !c.is_forbidden(duel)
        && duel.is_existing_matching_card(|tc, duel| eqtcfilter(tc, duel, c), tp, LOCATION_MZONE, 0, 1, None)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| eqfilter(c, duel, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, 0, location as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let filter = aux::necro_valley_filter(|c, duel| eqfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(eq) = g.get_first() else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_matching_card(tp, |c, duel| eqtcfilter(c, duel, eq), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(eqtc) = g.get_first()
        && duel.equip(tp, eq, eqtc)?
    {
        duel.break_effect()?;
        duel.destroy(e.get_handler(duel), REASON_EFFECT)?;
    }
    Ok(())
}

pub fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_LOST_TARGET)
        && c.get_previous_equip_target(duel)
            .is_some_and(|ec| ec.is_location(duel, LOCATION_GRAVE))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_WARRIOR) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_GRAVE | LOCATION_REMOVED;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, location) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, location, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
