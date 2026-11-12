// Number C6: Chronomaly Chaos Atlandis
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 6387204;
pub const XYZ_NUMBER: u32 = 6;
const ATLANDIS: u32 = 9161357;

pub const LISTED_NAMES: &[u32] = &[ATLANDIS];

// In the Extra Deck, Xyz Summoned with Level 7 Normal Monsters, with the opponent's monsters to equip. The second effect needs
// Number 6: Chronomaly Atlandis as Xyz Material, which only a Chaos Xyz Change attaches.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6368038, MZONE),
    (6368039, MZONE),
    (6740720, MZONE),
    (24311372, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.get_lp(1 - a.tp) != 100
            && a.e
                .get_handler(duel)
                .get_overlay_group(duel)
                .is_exists(duel, |tc, duel| tc.is_code(duel, ATLANDIS), 1, None)
    });
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<3, 3>, lpcost])
    });
    e2.set_operation(duel, lpop);
    c.register_effect(duel, e2);
}

fn nodamage(duel: &mut Duel, a: &Args) {
    let tp = a.tp;
    if duel.get_flag_effect(tp, ID) > 0 {
        return;
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_change_controler(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    Ok(false)
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, Some(ID), None)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, 1000);
    tc.register_effect(duel, e1);
    Ok(())
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
    {
        equipop(duel, c, e, a.tp, tc)?;
    }
    nodamage(duel, a);
    Ok(())
}

fn lpcostfilter(c: CardId, duel: &Duel) -> bool {
    c.has_flag_effect(duel, ID) && c.is_set_card(duel, &[SET_NUMBER]) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn lpcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let eqg = a.e.get_handler(duel).get_equip_group(duel).filter(duel, lpcostfilter, None);
    if chk == 0 {
        return Ok(!eqg.is_empty());
    }
    duel.send_to_grave(eqg, REASON_COST)?;
    Ok(false)
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.set_lp(1 - a.tp, 100);
    nodamage(duel, a);
    Ok(())
}
