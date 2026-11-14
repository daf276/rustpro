// Supreme King Gate Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 14513273;

pub const LISTED_SERIES: &[u16] = &[
    SET_SUPREME_KING_GATE,
    SET_PENDULUM_DRAGON,
    SET_XYZ_DRAGON,
    SET_SYNCHRO_DRAGON,
    SET_FUSION_DRAGON,
];
pub const LISTED_NAMES: &[u32] = &[ID, CARD_ZARC];
// Only the Gate monsters Zero and Infinity (unported) satisfy the effects, and no ported card mentions Supreme King Z-ARC.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (ID, HAND), (ID, DECK)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_REMOVE);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_player_card_target(duel, |duel, _, c, _, _| c.is_has_effect(duel, ID).is_some());
    c.register_effect(duel, e1);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, ID);
    e1a.set_range(duel, LOCATION_PZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, 0);
    e1a.set_target_card(duel, |duel, _, c| c.is_code(duel, CARD_ZARC));
    c.register_effect(duel, e1a);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUPREME_KING_GATE]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_code(duel, ID) && !c.is_forbidden(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(plfilter, a.tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || duel.destroy(c, REASON_EFFECT)? == 0 || !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUPREME_KING_GATE]) && !c.is_code(duel, ID)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_PZONE, 0, 1, None)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PENDULUM_DRAGON, SET_XYZ_DRAGON, SET_SYNCHRO_DRAGON, SET_FUSION_DRAGON]) && c.is_able_to_grave(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND | LOCATION_EXTRA, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND | LOCATION_EXTRA, 0, 1, 1, Some(c.into()))?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && c.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_ZARC) && !c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
