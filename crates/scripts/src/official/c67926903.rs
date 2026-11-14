// CXyz Barian Hope
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 67926903;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER];
// In the Extra Deck, Xyz Summoned on Number C101, C104 or C107, with Numbers with effects in the GY to copy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (12744567, MZONE),
    (49456901, MZONE),
    (88177324, MZONE),
    (48739166, GRAVE),
    (2061963, GRAVE),
    (63746411, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 1000));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, copytg);
    e2.set_operation(duel, copyop);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    let Some(no) = crate::official::xyz_number(c.get_code(duel)) else {
        return false;
    };
    c.is_faceup(duel) && (101..=107).contains(&no) && c.is_set_card_full(duel, &[SET_NUMBER_C], Some(lc), SUMMON_TYPE_XYZ as u64, tp)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_NUMBER]) && c.is_type(duel, TYPE_EFFECT)
}

fn copytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn copyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let code = tc.get_original_code(duel);
        let reset = RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, code as i32);
        e1.set_label(duel, tp.into());
        e1.set_reset(duel, reset);
        c.register_effect(duel, e1);
        let cid = c.copy_effect(duel, code, reset, 1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, aux::stringid(ID, 2));
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_PHASE | PHASE_END);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_count_limit(duel, 1, 0);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_condition(duel, resetcon);
        e2.set_operation(duel, resettop);
        e2.set_label(duel, cid.into());
        e2.set_label_object(duel, Some(e1));
        e2.set_reset(duel, reset);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn resetcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel)
        .is_some_and(|e1| duel.is_turn_player(1 - e1.get_label(duel) as u8))
}

fn resettop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let cid = e.get_label(duel) as u32;
    c.reset_effect(duel, cid, RESET_COPY);
    c.reset_effect(duel, RESET_DISABLE, RESET_EVENT);
    if let Some(e1) = e.get_label_object(duel) {
        e1.reset(duel);
    }
    duel.hint_selection(c);
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    Ok(())
}
