// Aegaion the Sea Castrum
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 10678778;

// In the Extra Deck, Xyz Summoned with Level 8 Normal Monsters (Black Luster Soldier), with an Xyz Monster of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (10000100, MZONE),
    (1269512, MZONE | OPPONENT),
    (1269512, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<8, 2>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rmfilter, tp, 0, LOCATION_EXTRA, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let g = duel.get_matching_group(rmfilter, tp, 0, LOCATION_EXTRA, None);
    if g.is_empty() {
        return Ok(());
    }
    let Some(tc) = g.random_select(duel, tp, 1).get_first() else {
        return Ok(());
    };
    duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    let atk = tc.get_attack(duel).max(0);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, atk);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn extra_type(c: CardId, duel: &Duel) -> u32 {
    c.get_type(duel) & (TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ)
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let ctype = extra_type(c, duel);
    c.is_faceup(duel)
        && ctype != 0
        && c.is_able_to_extra(duel)
        && duel.is_existing_matching_card(aux::faceup_filter(|c, duel| c.is_type(duel, ctype)), tp, 0, LOCATION_MZONE, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, tp), tp, 0, LOCATION_REMOVED, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, tp), tp, 0, LOCATION_REMOVED, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    let ctype = g.and_then(|g| g.get_first()).map_or(0, |tc| extra_type(tc, duel));
    let dg = duel.get_matching_group(aux::faceup_filter(|c, duel| c.is_type(duel, ctype)), tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? != 0 {
        let ctype = extra_type(tc, duel);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(
            tp,
            aux::faceup_filter(|c, duel| c.is_type(duel, ctype)),
            tp,
            0,
            LOCATION_MZONE,
            1,
            1,
            None,
        )?;
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
