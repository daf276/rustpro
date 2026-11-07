// Galaxy-Eyes Photon Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 93717133;

// In the hand, Special Summoned by Tributing two Normal Monsters with 2000 or more ATK, and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (89631139, MZONE),
    (39111158, MZONE),
    (89631139, MZONE),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_hint_timing(duel, TIMING_BATTLE_PHASE, TIMING_BATTLE_PHASE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, |tc, duel| tc.is_attack_above(duel, 2000), 2, release_options(c, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, |tc, duel| tc.is_attack_above(duel, 2000), (2, 2), true, release_options(c, tp))?;
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

fn rmcon(duel: &Duel, a: &Args) -> bool {
    duel.is_battle_phase() && !a.e.get_handler(duel).is_status(duel, STATUS_CHAINING)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    let (e, c) = (a.e, a.e.get_handler(duel));
    let bc = c.get_battle_target(duel);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| {
            bc.is_on_field(duel) && bc.is_can_be_effect_target(duel, e) && c.is_able_to_remove(duel) && bc.is_able_to_remove(duel)
        }));
    }
    let Some(bc) = bc else { return Ok(false) };
    duel.set_target_card(bc);
    let mut g = CardSet::from(c);
    g.insert(bc);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mut g = CardSet::from(c);
    g.insert(tc);
    let mut mcount = if tc.is_faceup(duel) { tc.get_overlay_count(duel) } else { 0 };
    if duel.remove(g, 0, REASON_EFFECT | REASON_TEMPORARY)? != 0 {
        let og = duel.get_operated_group();
        if !og.contains(tc) {
            mcount = 0;
        }
        for oc in og {
            oc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
        e1.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
        e1.set_label(duel, mcount as i64);
        e1.set_count_limit(duel, 1, 0);
        e1.set_label_object_group(duel, Some(og));
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = e.get_label_object_group(duel).unwrap_or_default();
    let sg = g.filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
    for tc in sg {
        let owner = e.get_owner(duel);
        if duel.return_to_field(tc)? && tc == owner && tc.is_faceup(duel) && e.get_label(duel) != 0 {
            let e1 = duel.create_effect(owner);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            e1.set_value(duel, e.get_label(duel) as i32 * 500);
            owner.register_effect(duel, e1);
        }
    }
    Ok(())
}
