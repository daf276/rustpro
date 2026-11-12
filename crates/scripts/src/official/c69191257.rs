// Jade Dragon Mech
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 69191257;

// In the Extra Deck, Synchro Summoned with Ally Mind (a Level 5 Tuner) and Kojikocy (Level 4), with Ally Mind in the Deck and Graveyard,
// and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (40155554, MZONE),
    (40155554, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (40155554, DECK),
    (40155554, DECK),
    (40155554, GRAVE),
    (53129443, HAND | OPPONENT),
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
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgsptg);
    e1.set_operation(duel, tgspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, descon);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn difflvfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel) && c.is_faceup(duel) && !c.is_level(duel, &[c.get_original_level(duel)])
}

fn tgspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_extra_chk: bool) -> bool {
    c.is_race(duel, RACE_MACHINE)
        && c.is_type(duel, TYPE_TUNER)
        && (c.is_able_to_grave(duel) || (mmz_extra_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn mmz_extra_chk(duel: &Duel, tp: u8) -> bool {
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(difflvfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn tgsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_extra_chk = mmz_extra_chk(duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tgspfilter(c, duel, e, tp, mmz_extra_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    if !mmz_extra_chk {
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    } else {
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_extra_chk = mmz_extra_chk(duel, tp);
    let hint = if mmz_extra_chk { aux::stringid(ID, 2) } else { HINTMSG_TOGRAVE };
    duel.hint(HINT_SELECTMSG, tp, hint);
    let g = duel.select_matching_card(
        tp,
        |c, duel| tgspfilter(c, duel, e, tp, mmz_extra_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    let mut op = Some(1);
    if mmz_extra_chk {
        let b1 = sc.is_able_to_grave(duel);
        let b2 = sc.is_can_be_special_summoned(duel, e, 0, tp, false, false);
        op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    }
    match op {
        Some(1) => {
            duel.send_to_grave(sc, REASON_EFFECT)?;
        }
        Some(2) => {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        _ => {}
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_synchro_summoned(duel)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_EFFECT)
}

fn descostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_able_to_remove_as_cost(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(descostfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let max_target_count = duel.get_target_count(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, descostfilter, tp, LOCATION_GRAVE, 0, 1, max_target_count as u8, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    a.e.set_label(duel, g.len() as i64);
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let target_count = a.e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(
        tp,
        aux::TRUE,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        target_count,
        target_count,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
