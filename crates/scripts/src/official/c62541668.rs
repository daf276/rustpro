// Number 77: The Seven Sins
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 62541668;
pub const XYZ_NUMBER: u32 = 77;

// In the Extra Deck, Xyz Summoned with two Level 12 Blue-Eyes Ultimate Dragons, with the opponent's Cyber Dragons to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (23995346, MZONE),
    (23995346, MZONE),
    (70095154, HAND | OPPONENT),
    (70095154, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op(duel: &mut Duel, e: EffectId, _: u8, _: CardId) -> Script<bool> {
        let c = e.get_handler(duel);
        c.register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TOFIELD), 0, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let rk = c.get_rank(duel);
    c.is_faceup(duel)
        && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && (rk == 10 || rk == 11)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_special_summoned(duel) && c.is_able_to_remove(duel) && c.is_monster(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(a.e.get_handler(duel).get_flag_effect(duel, ID) == 0
            && duel.is_existing_matching_card(rmfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(rmfilter, tp, 0, LOCATION_MZONE, None);
    let count = g.len() as u8;
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), count, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(rmfilter, tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 0 || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let og = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), None);
    if !og.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
        let sg = og.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.overlay(c, sg)?;
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
        return Ok(true);
    }
    Ok(false)
}
