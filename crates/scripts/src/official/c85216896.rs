// Zefra Metaltron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 85216896;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, _lc: CardId, _sumtype: u64, _tp: u8) -> bool {
        c.is_summon_location(duel, LOCATION_EXTRA)
    }
}

// Only bottom Link Markers: it links from an Extra Monster Zone after a Link Summon with monsters from the Extra Deck, which no SUPPORT
// card gets there, so the first effect (a linked monster of the Extra Deck being destroyed) is only reported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_LEAVE_FIELD_P);
    e1.set_condition(duel, regcon);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn thcfilter(c: CardId, duel: &Duel, lc: CardId, tp: u8) -> bool {
    (c.is_reason(duel, REASON_BATTLE) || (c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp))
        && lc.get_linked_group(duel).is_contains(c)
        && c.is_summon_location(duel, LOCATION_EXTRA)
        && c.is_location(duel, LOCATION_MZONE)
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    a.eg.unwrap_or_default()
        .is_exists(duel, |tc, duel| thcfilter(tc, duel, c, tp), 1, Some(c.into()))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (!c.is_location(duel, LOCATION_EXTRA) || (c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM)))
        && c.is_monster(duel)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE | LOCATION_EXTRA, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_GRAVE | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_can_be_effect_target(duel, e) && c.is_summon_location(duel, LOCATION_EXTRA) && c.is_able_to_remove(duel)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let rg = duel.get_matching_group(
        |c, duel| rmfilter(c, duel, e),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        Some(e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, rg, 2, 2, &|duel, sg, _, _| {
            rescon(duel, sg, tp)
        }));
    }
    let tg = aux::select_unselect_group(duel, rg, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp), tp, HINTMSG_REMOVE)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tg), 2, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.len() != 2 {
        return Ok(());
    }
    for tc in g {
        if duel.remove(tc, 0, REASON_EFFECT | REASON_TEMPORARY)? != 0 {
            tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_label_object_group(duel, Some(g));
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition_mut(duel, retcon);
    e1.set_operation(duel, retop);
    duel.register_effect(e1, tp);
    Ok(())
}

fn retfilter(c: CardId, duel: &Duel) -> bool {
    c.get_flag_effect(duel, ID) != 0
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    let e = a.e;
    if !e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .is_exists(duel, retfilter, 1, None)
    {
        e.reset(duel);
        return false;
    }
    true
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_label_object_group(duel).unwrap_or_default().filter(duel, retfilter, None);
    for tc in g {
        duel.return_to_field(tc)?;
    }
    Ok(())
}
