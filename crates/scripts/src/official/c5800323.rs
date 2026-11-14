// Poplar of the White Forest
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 5800323;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Two copies in the Extra Deck, Synchro Summoned from Galaxy Serpent and Astellar of the White Forest on the field, with a copy on the
// field and Silvera, Wolf Tamer of the White Forest in the Extra Deck to Synchro Summon with it, and two Silvy of the White Forest in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (11066358, MZONE),
    (25592142, MZONE),
    (25592142, MZONE),
    (41924516, EXTRA),
    (98385955, GRAVE),
    (98385955, GRAVE),
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
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_main_phase() && duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, synchtg);
    e2.set_operation(duel, synchop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    let p = c.get_owner(duel);
    c.is_set_card(duel, &[SET_WHITE_FOREST])
        && c.is_monster(duel)
        && c.check_unique_on_field_full(duel, p, LOCATION_SZONE)
        && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) >= 2 && duel.is_existing_target(plfilter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_target(tp, plfilter, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 2, tp, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    if ft <= 0 {
        return Ok(());
    }
    if tg.len() as i32 > ft {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        tg = tg.select(duel, tp, ft as u8, ft as u8, None)?;
    }
    for tc in tg {
        if duel.move_to_field(tc, tp, tc.get_owner(duel), LOCATION_SZONE, POS_FACEUP, tc.is_monster_card(duel))? {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_TYPE);
            e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
            e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn synchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(
            |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_controler(duel, 1 - tp) || !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), None)?;
        }
    }
    Ok(())
}
