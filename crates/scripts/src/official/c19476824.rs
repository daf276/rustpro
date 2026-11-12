// Metaphys Ragnarok
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 19476824;

// In the hand and on the field with Metaphys Armed Dragon in the Deck to Special Summon after battle damage, dealt to the opponent's weak Sangans.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (89189982, DECK),
    (89189982, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |_, a| a.ep != a.tp);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_decktop_group(tp, 3);
    if chk == 0 {
        return Ok(rg.filter_count(duel, CardId::is_able_to_remove, None) == 3);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), 3, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_decktop_group(tp, 3);
    if !g.is_empty() && duel.remove(g, POS_FACEUP, REASON_EFFECT)? != 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let og = duel.get_operated_group();
        let oc = og.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_METAPHYS]), None);
        if oc == 0 {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, oc as i32 * 300);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_METAPHYS]) && c.is_level_above(duel, 5) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let tc = g.get_first();
    if let Some(tc) = tc
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
        let e2 = duel.create_effect(e.get_handler(duel));
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_PHASE + PHASE_END);
        e2.set_count_limit(duel, 1, 0);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        let turn = i64::from(duel.get_turn_count()) + 1;
        e2.set_label(duel, turn);
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_condition_mut(duel, descon);
        e2.set_operation(duel, desop);
        duel.register_effect(e2, tp);
    }
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    if tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0) {
        i64::from(duel.get_turn_count()) == a.e.get_label(duel)
    } else {
        a.e.reset(duel);
        false
    }
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
