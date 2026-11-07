// Number F0: Utopic Future
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 65305468;
pub const XYZ_NUMBER: u32 = 0;

// In the Extra Deck, Xyz Summoned with two Evilswarm Exciton Knights, and Raigeki and Dark Hole in the opponent's hand to
// destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (46772449, MZONE),
    (46772449, MZONE),
    (46772449, MZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// The control effect needs a battle, which is too rare to assert; the destruction replacement is no chain link.
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID), Check::Report(&Check::Resolves)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(xyzfilter);
    const LV: Option<u32> = None;
    const CT: usize = 2;

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        let mg = g.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EQUIP_SPELL_XYZ_MAT).is_none(), None);
        mg.get_class_count(duel, |c, duel| i64::from(c.get_rank(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_NO_BATTLE_DAMAGE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e5.set_value(duel, 1);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_CONTROL);
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_DAMAGE_STEP_END);
    e6.set_target(duel, cttg);
    e6.set_operation(duel, ctop);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e7.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e7.set_code(duel, EFFECT_DESTROY_REPLACE);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_target(duel, reptg);
    c.register_effect(duel, e7);
}

fn xyzfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(xyz), sumtype, tp) && !c.is_set_card_full(duel, &[SET_NUMBER], Some(xyz), sumtype, tp)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_relate_to_battle(duel) && tc.is_controler_can_be_changed(duel)));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel)
        && tc.is_relate_to_battle(duel)
    {
        duel.get_control_full(tc, a.tp, PHASE_BATTLE as u16, 1, 0xff, duel.get_reason_player())?;
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && c.is_reason(duel, REASON_EFFECT)
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    Ok(true)
}
