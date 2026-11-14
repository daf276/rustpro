// Dingirsu, the Orcust of the Evening Star
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 93854893;

pub const LISTED_SERIES: &[u16] = &[SET_ORCUST];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned with two Blue-Eyes White Dragons or with Galatea-I on the field, with Girsu banished to
// attach, and the opponent's Raigeki and monsters to destroy it or send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (89631139, MZONE),
    (89631139, MZONE),
    (48835607, MZONE),
    (69811710, REMOVED),
    (69811710, REMOVED),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// The destruction replacement is no chain link; its prompt is the MSG_SELECT_EFFECTYN, counted for any card.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_sp_summon_once(duel, ID);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_type_full(duel, TYPE_LINK, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_set_card_full(duel, &[SET_ORCUST], Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a
            .eg
            .is_some_and(|eg| eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None))
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
        return Ok(true);
    }
    Ok(false)
}

fn matfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_faceup(duel) && c.is_can_be_xyz_material(duel, None, tp, REASON_EFFECT)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_ONFIELD, 1, None);
    let b2 = e.get_handler(duel).is_type(duel, TYPE_XYZ)
        && duel.is_existing_matching_card(|c, duel| matfilter(c, duel, tp), tp, LOCATION_REMOVED, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOGRAVE);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    } else if op == Some(2) {
        e.set_category(duel, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let g = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection_full(g, true);
                duel.send_to_grave(g, REASON_EFFECT)?;
            }
        }
        2 => {
            let c = e.get_handler(duel);
            if !(c.is_relate_to_effect(duel, e) && c.is_type(duel, TYPE_XYZ)) {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
            let g = duel.select_matching_card(tp, |tc, duel| matfilter(tc, duel, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection_full(g, true);
                duel.overlay(c, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}
