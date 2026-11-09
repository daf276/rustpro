// Borrelsword Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 85289965;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Link Summoned in the Extra Deck with three Ash Blossom & Joyous Spring (Effect Monsters).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (14558127, MZONE), (14558127, MZONE), (14558127, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_ATTACK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_battle_target(duel).is_some_and(|tc| tc.is_faceup(duel))
    });
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    duel.set_chain_limit(|_, _, ep, tp, _| tp == ep, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_DEFENSE, POS_FACEUP_DEFENSE],
            false,
        )?;
    }
    if c.is_relate_to_effect(duel, e) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_EXTRA_ATTACK);
        e2.set_value(duel, 1);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel) {
        tc.create_effect_relation(duel, a.e);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let tc = c.get_battle_target(duel);
    let Some(tc) = tc.filter(|tc| tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let half = (tc.get_attack(duel) + 1).div_euclid(2);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e2.set_value(duel, half);
    c.register_effect(duel, e2);
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, half);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
