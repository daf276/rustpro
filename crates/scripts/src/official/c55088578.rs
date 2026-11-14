// Onomatokage
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 55088578;

pub const LISTED_SERIES: &[u16] = &[SET_ZUBABA, SET_GAGAGA, SET_GOGOGO, SET_DODODO];

const ZUBABA_GROUP: &[u16] = &[SET_ZUBABA, SET_GAGAGA, SET_GOGOGO, SET_DODODO];

// Dodododwarf Gogogoglove on the field to enable the Special Summon, and an Xyz Monster (Odd-Eyes Absolute Dragon) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (59724555, MZONE), (16691074, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.add_setcodes_rule(duel, ID, true, &[SET_ZUBABA]);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, textg);
    e2.set_operation(duel, texop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, ZUBABA_GROUP)),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        super::c59724555::banish_when_leaving(duel, c, c);
    }
    Ok(())
}

fn texfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_able_to_extra(duel)
}

fn textg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(texfilter, tp, LOCATION_GRAVE, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn texop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, texfilter, tp, LOCATION_GRAVE, 0, 1, 2, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
