// Tellarknight Cygnian
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 60700283;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT, SET_CONSTELLAR];

// Satellarknight Deneb and Unukalhai in the Deck to search, and Deneb (LIGHT) on the field to raise its Level.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (75878039, DECK),
    (1050186, DECK),
    (1050186, DECK),
    (75878039, MZONE),
    (75878039, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e1c = e1a.clone_effect(duel);
    e1c.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DOUBLE_XYZ_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_value(duel, 1);
    e3.set_xyz_material_operation(duel, |duel, _, xyz, _, minxyzct| {
        minxyzct >= 3 && xyz.is_set_card(duel, &[SET_TELLARKNIGHT])
    });
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.has_level(duel) && c.is_faceup(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc != c && lvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, a.e) && tc.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
