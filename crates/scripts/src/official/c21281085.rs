// Fabled Levilazebul
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 21281085;

pub const LISTED_SERIES: &[u16] = &[SET_FABLED];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned (Level 10) with Angel Trumpeter (Level 4 Tuner), Peacock (Level 5) and Key Mace (Level 1), with Fabled
// Lurrie to Tribute and monsters of the opponent to take, and a copy and Fabled Lurrie in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (87979586, MZONE),
    (20624263, MZONE),
    (1929294, MZONE),
    (97651498, MZONE),
    (97651498, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
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
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn rlfilter(c: CardId, duel: &Duel, tp: u8, chk: i32) -> bool {
    c.is_releasable_by_effect(duel)
        && c.is_set_card(duel, &[SET_FABLED])
        && (chk == 1 || duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0)
}

fn controlfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_change_controler(duel)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| rlfilter(c, duel, tp, chk), tp, LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(controlfilter, tp, 0, LOCATION_MZONE, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let cg = duel.get_matching_group(controlfilter, tp, 0, LOCATION_MZONE, None);
    if cg.is_empty() {
        return Ok(());
    }
    let rg = duel.get_matching_group(|c, duel| rlfilter(c, duel, tp, 1), tp, LOCATION_MZONE, 0, None);
    if rg.is_empty() {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        (
            duel.get_mzone_count_full(tp, Some(sg), tp, LOCATION_REASON_CONTROL, 0xff) >= sg.len() as i32,
            false,
        )
    };
    let g = aux::select_unselect_group(duel, rg, 1, cg.len(), &rescon, tp, HINTMSG_RELEASE)?;
    if g.is_empty() || duel.release(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let og = duel.get_operated_group();
    let ct = og.len();
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let sg = cg.difference(og).select(duel, tp, ct as u8, ct as u8, None)?;
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(sg);
    if duel.get_control(sg, tp)? > 0 {
        for tc in duel.get_operated_group() {
            tc.negate_effects(duel, c, 0, false);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FABLED]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(c.is_able_to_extra(duel) && duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_EXTRA)
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
