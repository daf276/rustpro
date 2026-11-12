// Unchained Enma Lord Yama
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 94014327;
pub const MATERIAL_SETCODE: &[u16] = &[SET_UNCHAINED_SOUL];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_UNCHAINED_SOUL], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// On the field with Unchained Soul of Rage and Aruha, Link Summoned from them, and Dark Hole in either hand to destroy the Fiends
// to revive and Yama itself.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (67680512, MZONE),
    (26236560, MZONE),
    (53129443, HAND),
    (53129443, HAND | OPPONENT),
];
// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, desreptg);
    e2.set_operation(duel, desrepop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, turn_count: i32) -> bool {
    c.is_race(duel, RACE_FIEND)
        && c.is_reason(duel, REASON_DESTROY)
        && c.get_turn_id(duel) == turn_count
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let turn_count = duel.get_turn_count();
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp, turn_count));
    }
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp, turn_count), tp, LOCATION_GRAVE, 0, 1, None));
    }
    ft = ft.min(2);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp, turn_count),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        ft as u8,
        None,
    )?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, count as u8, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() || (tg.len() == 2 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()) {
        return Ok(());
    }
    if tg.len() == 2 && duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        // The Lua only assigns the selected card to `tg`, so it is never summoned.
        tg.filter_select(
            duel,
            tp,
            |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            1,
            1,
            None,
        )?;
    } else {
        for tc in tg {
            if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
                tc.register_flag_effect_full(
                    duel,
                    ID,
                    RESET_EVENT | RESETS_STANDARD,
                    EFFECT_FLAG_CLIENT_HINT,
                    1,
                    0,
                    aux::stringid(ID, 1),
                );
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_type(duel, EFFECT_TYPE_FIELD);
                e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
                e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
                e1.set_range(duel, LOCATION_MZONE);
                e1.set_absolute_range(duel, tp, 1, 0);
                e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_controler(duel, a.e.get_owner_player(duel)));
                e1.set_target_card(duel, |duel, _, c| c.is_race_except(duel, RACE_FIEND));
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
            }
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_destructable_full(duel, e) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && duel.is_existing_matching_card(
                |tc, duel| repfilter(tc, duel, e),
                tp,
                LOCATION_ONFIELD,
                LOCATION_ONFIELD,
                1,
                Some(c.into()),
            ));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| repfilter(tc, duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        1,
        Some(c.into()),
    )?;
    duel.hint_selection(g);
    e.set_label_object_group(duel, Some(g));
    if let Some(tc) = g.get_first() {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
    }
    Ok(true)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(tc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
