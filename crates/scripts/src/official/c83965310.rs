// Destiny HERO - Plasma
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 83965310;

// Three copies of Kojikocy (a Normal Monster) to Tribute and one of the opponent's to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_EQUIP);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, eqcon);
    e3.set_target(duel, eqtg);
    e3.set_operation(duel, eqop);
    c.register_effect(duel, e3);
    aux::add_er_equip_limit(duel, c, Some(eqcon), e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_DISABLE);
    c.register_effect(duel, e4);
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, aux::TRUE, 3, release_options(c, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, aux::TRUE, (3, 3), true, release_options(c, tp))?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.get_flag_effect(duel, ID) != 0
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_equip_group(duel).filter(duel, eqfilter, None).is_empty()
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
    let atk = if tc.is_facedown(duel) {
        0
    } else {
        (tc.get_text_attack(duel) / 2).max(0)
    };
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, Some(ID), None)? {
        return Ok(());
    }
    if atk > 0 {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_OWNER_RELATE);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e2.set_value(duel, atk);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        equipop(duel, e.get_handler(duel), e, tp, tc)?;
    }
    Ok(())
}
