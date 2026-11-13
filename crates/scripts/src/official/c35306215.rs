// Condemned Darklord
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 35306215;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FAIRY, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from two Fairies, with Level 8 Fairies in the hand to Summon without Tributes, monsters in the Graveyard to banish and
// Darklords to search; the Summon of another card counts for no check.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (38142739, MZONE),
    (38142739, MZONE),
    (38142739, MZONE),
    (38142739, GRAVE),
    (38142739, GRAVE),
    (38142739, GRAVE),
    (64413994, HAND),
    (64413994, HAND),
    (10426067, DECK),
    (84031359, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Report(&Check::TributeSummoned(0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_condition(duel, otcon);
    e1.set_target_field_proc(duel, ottg, sumtg);
    e1.set_operation(duel, otop);
    e1.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_PROC);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, thcost);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_RECOVER);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e4.set_target(duel, rectg);
    e4.set_operation(duel, recop);
    c.register_effect(duel, e4);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn otcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    a.min <= 2
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(rmfilter, tp, LOCATION_GRAVE, 0, 2, None)
}

fn ottg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    let (mi, ma) = c.get_tribute_requirement(duel);
    mi <= 2 && ma >= 2 && c.is_race(duel, RACE_FAIRY)
}

fn sumtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card_full(tp, rmfilter, tp, LOCATION_GRAVE, 0, 2, 2, true, None)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn otop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sg) = a.e.get_label_object_group(duel) {
        duel.remove(sg, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD]) && c.is_monster(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.send_to_grave(tc, REASON_EFFECT).map(drop),
            574,
        )?;
    }
    Ok(())
}

fn fairy_count(duel: &Duel, tp: u8) -> u32 {
    duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FAIRY)),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    ) as u32
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let ct = fairy_count(duel, tp);
    duel.set_target_player(tp);
    duel.set_target_param(ct * 500);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, (ct * 500) as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = fairy_count(duel, tp);
    duel.recover(tp, ct * 500, REASON_EFFECT)?;
    Ok(())
}
