// Adamancipator Risen - Raptite
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT};

const ID: u32 = 73079836;

pub const LISTED_SERIES: &[u16] = &[SET_ADAMANCIPATOR];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field with Rock Normal Monsters (Destroyer Golem) in the Deck, a WIND monster in the Graveyard and cards in the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (73481154, DECK),
    (73481154, DECK),
    (73481154, DECK),
    (ID, GRAVE),
    (73481154, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, LOCATION_DECK, 0) >= 5);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) == 0 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 5)?;
    let g = duel.get_decktop_group(tp, 5).filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    let mut ct = 0;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.disable_shuffle_check(true);
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        ct = 1;
    }
    let ac = 5 - ct;
    duel.move_to_deck_bottom_count(ac, tp)?;
    duel.sort_deckbottom(tp, tp, ac as u16)
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && duel.is_existing_matching_card(|c, duel| c.is_attribute(duel, ATTRIBUTE_WIND), a.tp, LOCATION_GRAVE, 0, 1, None)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
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
