// Fire King Sanctuary
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 65305978;
const FIRE_KING_ISLAND: u32 = 57554544;

pub const LISTED_NAMES: &[u32] = &[FIRE_KING_ISLAND];
pub const LISTED_SERIES: &[u16] = &[SET_FIRE_KING];
// In hand and on the field, Fire King Island in the Deck to place, and two Fire King Courtier Ulcanix (Level 4) on the
// field for Number 60: Dugares the Timeless in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (FIRE_KING_ISLAND, DECK),
    (FIRE_KING_ISLAND, DECK),
    (44455560, MZONE),
    (44455560, MZONE),
    (66011101, EXTRA),
    (66011101, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, desreptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, desrepop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None)
    });
    e3.set_target(duel, xyztg);
    e3.set_operation(duel, xyzop);
    c.register_effect(duel, e3);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(
        |c, duel| c.is_code(duel, FIRE_KING_ISLAND) && !c.is_forbidden(duel),
        tp,
        LOCATION_DECK,
        0,
        None,
    );
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        duel.place_field_spell(tc, tp)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_location(duel, LOCATION_FZONE)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_destructable_full(duel, e) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(cfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, None);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && g.is_exists(duel, |c, duel| desfilter(c, duel, e), 1, None));
    }
    let c = e.get_handler(duel);
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let sg = g.filter_select(duel, tp, |c, duel| desfilter(c, duel, e), 1, 1, None)?;
    e.set_label_object_group(duel, Some(sg));
    duel.hint_selection_full(sg, true);
    if let Some(tc) = sg.get_first() {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
    }
    Ok(true)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(tc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_xyz_summonable(duel, None, Some(mg))
}

fn fire_kings(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_FIRE_KING])),
        tp,
        LOCATION_MZONE,
        0,
        None,
    )
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let mg = fire_kings(duel, tp);
    if chk == 0 {
        return Ok(!mg.is_empty() && duel.is_existing_matching_card(|c, duel| xyzfilter(c, duel, mg), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mg = fire_kings(duel, tp);
    if mg.is_empty() {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| xyzfilter(c, duel, mg), tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.xyz_summon(tp, sc, None, Some(mg))?;
        }
    }
    Ok(())
}
