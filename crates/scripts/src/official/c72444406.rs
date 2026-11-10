// Enigmaster Packbit
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72444406;

// Angel Trumpeter (a Tuner) and Hyosube to Synchro Summon it, a monster in each Graveyard and on the opponent's field to place, and
// other copies in the Graveyard to be placed as a Continuous Trap.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (ID, GRAVE),
    (87979586, MZONE),
    (2118022, MZONE),
    (2118022, MZONE),
    (2118022, HAND),
    (2118022, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_trap(duel));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    let owner = c.get_owner(duel);
    c.is_monster(duel)
        && c.is_faceup(duel)
        && duel.get_location_count(owner, LOCATION_SZONE) > 0
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, owner)
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(plfilter(chkc, duel)
            && ((chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE))
                || (chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE))));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_target(plfilter, tp, LOCATION_GRAVE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let tc = duel
        .select_target(tp, plfilter, tp, LOCATION_GRAVE, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first());
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    if let Some(tc) = tc.filter(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(tc.into()), 1, tp, 0);
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.discard_hand(tp, discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)? == 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    let owner = tc.get_owner(duel);
    if !tc.is_relate_to_effect(duel, e) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if duel.get_location_count(owner, LOCATION_SZONE) == 0 {
        duel.send_to_grave_full(tc, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
    } else if tc.check_unique_on_field(duel, owner)
        && duel.move_to_field(tc, tp, owner, LOCATION_SZONE, POS_FACEUP, tc.is_monster_card(duel))?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_TRAP | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_original_type(duel, TYPE_MONSTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_STZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_STZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_STZONE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
