// Infernoble Knight Emperor Charles
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 77656797;

// A copy on the field and two in the Extra Deck, Synchro Summoned with Infernoble Knight Oliver (a Level 4 Tuner) and Shapesnatch (Level 5), with
// Durendal to equip from the hand and Graveyard, and Burstinatrix (a FIRE Warrior) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (58346901, MZONE),
    (58346901, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (37478723, HAND),
    (37478723, GRAVE),
    (58932615, DECK),
    (58932615, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_EQUIP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    aux::add_er_equip_limit(duel, c, None, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_ONFIELD, LOCATION_ONFIELD) > 0);
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn eqsfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.check_equip_target(duel, ec)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|g, duel| eqsfilter(g, duel, c), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, 0, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn eqmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_FIRE) && !c.is_forbidden(duel)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let filter = aux::necro_valley_filter(|g, duel| eqsfilter(g, duel, c));
    let tc = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first();
    let Some(tc) = tc else { return Ok(()) };
    if !duel.equip(tp, tc, c)? {
        return Ok(());
    }
    let tg = duel.get_matching_group(eqmfilter, tp, LOCATION_DECK, 0, None);
    if !tg.is_empty() && duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let sg = tg.select(duel, tp, 1, 1, None)?;
        let Some(sc) = sg.get_first() else { return Ok(()) };
        duel.break_effect()?;
        equipop(duel, c, e, tp, sc)?;
    }
    Ok(())
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, Some(true))? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_OWNER_RELATE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, 500);
    tc.register_effect(duel, e1);
    Ok(())
}
