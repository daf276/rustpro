// Mind Castlin
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 12172567;

// In the Extra Deck, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Zanki (Level 5) against the opponent's Sangan,
// with the opponent's Smashing Ground in hand to destroy it, leaving a monster on each field to swap.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16449363, MZONE),
    (16449363, MZONE),
    (30090452, MZONE),
    (30090452, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (97169186, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, cttg1);
    e1.set_operation(duel, ctop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, ctcon);
    e2.set_target(duel, cttg2);
    e2.set_operation(duel, ctop2);
    c.register_effect(duel, e2);
}

fn ctfilter(c: CardId, duel: &Duel, tp: u8, e: Option<EffectId>) -> bool {
    c.is_faceup(duel)
        && c.is_able_to_change_controler(duel)
        && duel.get_mzone_count_full(c.get_controler(duel), Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0
        && e.is_none_or(|e| c.is_can_be_effect_target(duel, e))
}

fn cttg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && ctfilter(chkc, duel, tp, None));
    }
    if chk == 0 {
        return Ok(c.is_able_to_change_controler(duel)
            && duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0
            && duel.is_existing_target(|c, duel| ctfilter(c, duel, tp, None), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let mut g = duel
        .select_target(tp, |c, duel| ctfilter(c, duel, tp, None), tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(g), 2, tp, 0);
    Ok(false)
}

fn ctop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
    {
        duel.swap_control(c, tc, 0, 0)?;
    }
    Ok(())
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_synchro_summoned(duel)
}

fn cttg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| ctfilter(c, duel, tp, None), tp, LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_target(|c, duel| ctfilter(c, duel, tp, None), tp, 0, LOCATION_MZONE, 1, None),
        );
    }
    let g = duel.get_matching_group(|c, duel| ctfilter(c, duel, tp, Some(e)), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let tg = aux::select_unselect_group(
        duel,
        g,
        2,
        2,
        &aux::dpcheck(|c, duel| i64::from(c.get_controler(duel))),
        tp,
        HINTMSG_CONTROL,
    )?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(tg), 2, tp, 0);
    Ok(false)
}

fn ctop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    let mut cards = tg.into_iter();
    if let (Some(c1), Some(c2), 2) = (cards.next(), cards.next(), tg.len()) {
        duel.swap_control(c1, c2, 0, 0)?;
    }
    Ok(())
}
