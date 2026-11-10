// Galaxy-Eyes Tachyon Primal
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 45710945;

// On the field and in the hand, with Dark Rebellion Xyz Dragons (Level 4) of both players to attack and to attach to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (16195942, MZONE),
    (16195942, MZONE),
    (16195942, MZONE | OPPONENT),
    (16195942, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, attachcon);
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_hint_timing(
        duel,
        0,
        TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E,
    );
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE | EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn attachcon(duel: &Duel, a: &Args) -> bool {
    let Some(ac) = duel.get_attacker() else { return false };
    ac.is_race(duel, RACE_DRAGON)
        && ac.is_type(duel, TYPE_XYZ)
        && a.e.get_handler(duel).is_can_be_xyz_material(duel, Some(ac), a.tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(ac) = duel.get_attacker() {
        duel.set_target_card(ac);
    }
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(ac) = duel.get_attacker()
        && c.is_relate_to_effect(duel, a.e)
        && ac.is_relate_to_effect(duel, a.e)
    {
        duel.overlay_full(ac, c, true)?;
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            (duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
                || (duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
                    && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let b2 = duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    let target_player = if op == Some(1) { tp } else { 1 - tp };
    duel.special_summon(c, 0, tp, target_player, false, false, POS_FACEUP)?;
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let owner = a.e.get_handler(duel).get_owner(duel);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, owner, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TACHYON]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p = a.e.get_handler(duel).get_owner(duel);
    duel.hint(HINT_SELECTMSG, p, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(p, thfilter, p, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, p, REASON_EFFECT)?;
        duel.confirm_cards(1 - p, g)?;
    }
    Ok(())
}
