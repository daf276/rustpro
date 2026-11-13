// Red Dragon Archfiend's Chain
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 92936364;
const TUNERS_REVEALED: u8 = 0;

pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];

// Set and in the hand with Red Dragon Archfiend in the Extra Deck to reveal, Archfiend Heiress and Rokket Caliber (a Tuner) in the hand, and the opponent's Archfiend Heiress to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (66540884, HAND),
    (67127799, HAND),
    (66540884, MZONE | OPPONENT),
    (66540884, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e0.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    e0.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(extracostfilter, a.e.get_handler_player(duel), LOCATION_EXTRA, 0, 1, None)
    });
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_label_object(duel, Some(e0));
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, Persistent::target_filter);
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel) as i32)),
    );
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_DISABLE);
    c.register_effect(duel, e3);
}

fn handcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && !c.is_public(duel)
}

fn rescon(duel: &Duel, sg: CardSet, max_target_count: usize) -> (bool, bool) {
    (
        sg.filter_count(duel, |c, duel| c.is_type(duel, TYPE_TUNER), None) < max_target_count,
        false,
    )
}

fn extracostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND) && !c.is_public(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(label_obj) = e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    let hg = duel.get_matching_group(handcostfilter, tp, LOCATION_HAND, 0, None);
    let max_target_count = duel.get_target_count(aux::faceup_filter(CardId::is_effect_monster), tp, 0, LOCATION_MZONE, None);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, max_target_count);
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(!hg.is_empty() && max_target_count > 0 && aux::select_unselect_group_check(duel, hg, 1, hg.len(), &rescon));
    }
    if label_obj.get_label(duel) > 0 {
        label_obj.set_label(duel, 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, extracostfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_extra(tp)?;
    }
    let sg = aux::select_unselect_group_full(duel, hg, 1, hg.len(), &rescon, tp, HINTMSG_CONFIRM, Some(&rescon), false)?;
    duel.confirm_cards(1 - tp, sg)?;
    duel.shuffle_hand(tp)?;
    e.set_label(duel, sg.len() as i64 * -100);
    let tuners = sg.filter_count(duel, |c, duel| c.is_type(duel, TYPE_TUNER), None) + 1;
    e.set_chain_data(duel, TUNERS_REVEALED, tuners as i64);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_effect_monster(duel)
            && chkc.is_faceup(duel));
    }
    let filter = aux::faceup_filter(CardId::is_effect_monster);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let ct = e.get_chain_data(duel, TUNERS_REVEALED).unwrap_or(0) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if tg.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in tg {
        c.set_card_target(duel, tc);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_SELF_DESTROY);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_card_target(duel).is_empty());
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}
