// Hundred Eyes Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 95453143;

pub const LISTED_SERIES: &[u16] = &[SET_EARTHBOUND_IMMORTAL];
// In the Extra Deck, Synchro Summoned from Earthbound Prisoner Line Walker (Level 3 DARK Tuner) and Stone Sweeper (Level 5 Fiend);
// more Prisoners in the Graveyard to banish and copy, and Dark Hole to destroy it. No Earthbound Immortal is ported to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (19322865, MZONE),
    (19322865, MZONE),
    (72323266, MZONE),
    (72323266, MZONE),
    (46939151, GRAVE),
    (46939151, GRAVE),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_FIEND, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_SINGLE);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_DESTROY));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 6)
        && c.is_type(duel, TYPE_EFFECT)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let exception = Some(e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, exception)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    if let Some(tc) = g.get_first() {
        let code = tc.get_original_code(duel);
        e.set_label(duel, i64::from(code));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let code = e.get_label(duel) as u32;
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_value(duel, code as i32);
        c.register_effect(duel, e1);
        c.copy_effect(duel, code, RESETS_STANDARD_PHASE_END, 1);
    }
    Ok(())
}

fn thtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EARTHBOUND_IMMORTAL]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
