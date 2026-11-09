// Bystial Baldrake
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72656408;

// In the hand with banished-able LIGHT and DARK monsters (Prevent Rats) in the Graveyard and on the field, and Extra Deck
// monsters of the opponent's (Number 106: Giant Hand, Tornado Dragon) to Summon and banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (549481, GRAVE),
    (549481, GRAVE),
    (6983839, MZONE | OPPONENT),
    (6983839, MZONE | OPPONENT),
    (63746411, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !spquickcon(duel, a));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, spquickcon);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 1));
    e3a.set_category(duel, CATEGORY_REMOVE);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3a.set_code(duel, EVENT_CUSTOM + ID);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_count_limit(duel, 1, (ID, 1));
    e3a.set_cost(duel, rmcost);
    e3a.set_target(duel, rmtg);
    e3a.set_operation(duel, rmop);
    c.register_effect(duel, e3a);
    e3a.set_label_object_group(duel, Some(CardSet::default()));
    let e3b = duel.create_effect(c);
    e3b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3b.set_range(duel, LOCATION_MZONE);
    e3b.set_label_object(duel, Some(e3a));
    e3b.set_operation(duel, regop);
    c.register_effect(duel, e3b);
}

fn spquickcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0
}

fn spfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
        && c.is_able_to_remove(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && c.is_faceup(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && spfilter(chkc, duel, tp));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c, duel| spfilter(c, duel, tp),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            None,
        ) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        LOCATION_GRAVE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_REMOVED)
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8, e: Option<EffectId>) -> bool {
    c.is_summon_player(duel, 1 - tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_able_to_remove(duel)
        && c.is_type(duel, TYPE_RITUAL | TYPE_EXTRA)
        && e.is_none_or(|e| c.is_can_be_effect_target(duel, e))
}

fn summoned_group(duel: &Duel, e: EffectId) -> CardSet {
    e.get_label_object_group(duel).unwrap_or_default()
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let sg = summoned_group(duel, e).filter(duel, |tc, duel| tgfilter(tc, duel, tp, Some(e)), None);
    let attributes = |tc: CardId, duel: &Duel| tc.is_attribute(duel, ATTRIBUTE_DARK | ATTRIBUTE_LIGHT);
    // aux.ReleaseCheckTarget: a card of sg stays that the release does not take.
    let check = |duel: &Duel, released: CardSet, _: u8, _: CardSet| (sg.is_exists(duel, aux::TRUE, 1, Some(released)), false);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, attributes, (1, 1), false, Some(&check), Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, attributes, (1, 1), false, Some(&check), Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = summoned_group(duel, e).filter(duel, |tc, duel| tgfilter(tc, duel, tp, Some(e)), None);
    if let Some(chkc) = chkc {
        return Ok(g.contains(chkc) && tgfilter(chkc, duel, tp, None));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let tc = if g.len() == 1 {
        g
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        g.select(duel, tp, 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tc), 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_flag_effect(tp, ID) > 0 {
        return Ok(());
    }
    let tg = a.eg.unwrap_or_default().filter(duel, |tc, duel| tgfilter(tc, duel, tp, None), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(summoned) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = summoned_group(duel, summoned);
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    summoned.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    Ok(())
}
