// Orcustrion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, REMOVED};

const ID: u32 = 3134857;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_ORCUST], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Link Summoned from Orcust Cymbal Skeletons, with three more banished Machines to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (21441617, MZONE),
    (21441617, MZONE),
    (21441617, MZONE),
    (21441617, REMOVED),
    (21441617, REMOVED),
    (21441617, REMOVED),
];
// The Quick Effect needs Orcustrated Babel, which is not ported.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| {
        duel.is_player_affected_by_effect(a.tp, CARD_ORCUSTRATED_BABEL).is_none()
    });
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER);
    e4.set_condition(duel, |duel, a| {
        aux::stat_change_damage_step_condition(duel, a) && duel.is_player_affected_by_effect(a.tp, CARD_ORCUSTRATED_BABEL).is_some()
    });
    c.register_effect(duel, e4);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, 0, 3, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, 0, 3, 3, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let og = duel.get_operated_group();
    if og.is_empty() {
        return Ok(());
    }
    if og.is_exists(duel, |oc, duel| oc.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    let g = duel.get_matching_group(aux::faceup_filter(CardId::is_linked), tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.break_effect()?;
        for tc in g {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            tc.register_effect(duel, e2);
            if tc.is_type(duel, TYPE_TRAPMONSTER) {
                let e3 = e1.clone_effect(duel);
                e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
                tc.register_effect(duel, e3);
            }
            let e4 = e1.clone_effect(duel);
            e4.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e4.set_value(duel, 0);
            tc.register_effect(duel, e4);
            let e5 = e4.clone_effect(duel);
            e5.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
            tc.register_effect(duel, e5);
        }
    }
    Ok(())
}
