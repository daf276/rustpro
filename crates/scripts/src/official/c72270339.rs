// Diabellstar the Black Witch
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 72270339;

pub const LISTED_SERIES: &[u16] = &[SET_SINFUL_SPOILS];
// With Unchained Twins - Rakea on the field to destroy it, and Sinful Spoils Spells in the Deck to set.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (53624265, MZONE), (80845034, DECK), (66328392, DECK)];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spproccon);
    e1.set_target(duel, spproctg);
    e1.set_operation(duel, spprocop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, spgycon);
    e4.set_target(duel, spgytg);
    e4.set_operation(duel, spgyop);
    c.register_effect(duel, e4);
}

pub fn tgfilter(c: CardId, duel: &Duel, tp: u8, effect: bool) -> bool {
    let tg_check = if effect {
        c.is_able_to_grave(duel)
    } else {
        c.is_able_to_grave_as_cost(duel)
    };
    tg_check && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spproccon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let rg = duel.get_matching_group(
        |c, duel| tgfilter(c, duel, tp, false),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        Some(c.into()),
    );
    !rg.is_empty() && aux::select_unselect_group_check(duel, rg, 1, 1, &|_, _, _, _| (true, false))
}

fn spproctg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(
        |c, duel| tgfilter(c, duel, tp, false),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        a.c.map(CardSet::from),
    );
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TOGRAVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spprocop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SINFUL_SPOILS]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}

fn spgycon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_HAND | LOCATION_ONFIELD) && c.is_previous_controler(duel, a.tp) && duel.is_turn_player(1 - a.tp)
}

fn spgytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| tgfilter(c, duel, tp, true),
                tp,
                LOCATION_HAND | LOCATION_ONFIELD,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spgyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| tgfilter(c, duel, tp, true),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
