// Ashtraja Susanoo
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 17020474;
const BANISH_COUNT: u8 = 0;
const NUMBER_OF_TARGETS: u8 = 1;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp)
    }
}

// Synchro Summoned with Ashtra Divine Domain (a Tuner once Special Summoned) and Akasuna, with Traps in the Graveyard to banish,
// and Ashtra monsters and Traps in the Graveyard and banishment to add or Set in the End Phase.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (37839434, HAND),
    (37839434, MZONE),
    (37839434, GRAVE),
    (37839434, GRAVE),
    (37839434, REMOVED),
    (38331244, MZONE),
    (38331244, GRAVE),
    (1710647, REMOVED),
    (26202165, MZONE | OPPONENT),
    (26202165, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_cost(duel, bancost);
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SET + CATEGORY_LEAVE_GRAVE + CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thsettg);
    e2.set_operation(duel, thsetop);
    c.register_effect(duel, e2);
}

fn bancostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_able_to_remove_as_cost(duel)
}

fn bancost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(bancostfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let max_banish_count = duel.get_matching_group_count(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, bancostfilter, tp, LOCATION_GRAVE, 0, 1, max_banish_count as u8, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_chain_data(duel, BANISH_COUNT, g.len() as i64);
    Ok(false)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let count = e.get_chain_data(duel, BANISH_COUNT).unwrap_or(0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, count as u8, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let banish_count = e.get_chain_data(duel, BANISH_COUNT).unwrap_or(0) as usize;
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if g.len() < banish_count {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let sg = g.select(duel, tp, banish_count as u8, banish_count as u8, None)?;
    if sg.len() == banish_count {
        duel.hint_selection(sg);
        duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !(c.is_set_card(duel, &[SET_ASHTRA]) && c.is_faceup(duel)) {
        return false;
    }
    if c.is_monster(duel) {
        c.is_able_to_hand(duel)
            || (mmz_chk && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff))
    } else if c.is_trap(duel) {
        c.is_able_to_hand(duel) || c.is_ssetable(duel)
    } else {
        false
    }
}

fn rescon(to_extra_chk: bool, free_mzones: i32, free_stzones: i32) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        if sg.len() == 2 && !to_extra_chk {
            return (false, false);
        }
        let cannot_add_g = sg.filter(duel, |c, duel| !c.is_able_to_hand(duel), None);
        if cannot_add_g.len() < 2 {
            return (true, false);
        }
        let (monsters, traps) = cannot_add_g.split(duel, CardId::is_monster, None);
        (monsters.len() as i32 <= free_mzones && traps.len() as i32 <= free_stzones, false)
    }
}

fn thsettg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let filter = |c: CardId, duel: &Duel| setfilter(c, duel, e, tp, mmz_chk);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    let c = e.get_handler(duel);
    let to_extra_chk = c.is_able_to_extra(duel);
    let free_mzones = duel.get_location_count(tp, LOCATION_MZONE);
    let free_stzones = duel.get_location_count(tp, LOCATION_SZONE);
    let g = duel.get_target_group(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    let rescon = rescon(to_extra_chk, free_mzones, free_stzones);
    let tg = aux::select_unselect_group(duel, g, 1, 2, &rescon, tp, aux::stringid(ID, 2))?;
    duel.set_target_card(tg);
    duel.set_possible_operation_info(0, CATEGORY_SET, Some(tg), tg.len() as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(tg), tg.len() as u8, tp, 0);
    if tg.is_exists(duel, CardId::is_monster, 1, None) {
        let monsters = tg.filter(duel, CardId::is_monster, None);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(monsters), 1, tp, 0);
    }
    if tg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None) {
        let graveyard = tg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(graveyard), 1, tp, 0);
    }
    if tg.len() == 2 {
        duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, tp, 0);
    }
    e.set_chain_data(duel, NUMBER_OF_TARGETS, tg.len() as i64);
    Ok(false)
}

fn thsetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let mut success = false;
    for i in 1..=tg.len() {
        let tc = if tg.len() == 2 {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
            let Some(tc) = tg.select(duel, tp, 1, 1, None)?.get_first() else {
                continue;
            };
            duel.hint_selection(tc);
            tg.remove(tc);
            tc
        } else {
            let Some(tc) = tg.get_first() else { continue };
            if i == 2 {
                duel.hint_selection(tc);
            }
            tc
        };
        let monster_set_chk = tc.is_monster(duel)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff);
        let trap_set_chk = tc.is_trap(duel) && tc.is_ssetable(duel);
        let oper = |duel: &mut Duel| {
            if monster_set_chk {
                if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)? {
                    duel.confirm_cards(1 - tp, tc)?;
                    return Ok(true);
                }
            } else if trap_set_chk {
                return Ok(duel.sset(tp, tc)? > 0);
            }
            Ok(false)
        };
        let success_chk = aux::to_hand_or_else_result(duel, tc, tp, |_, _| monster_set_chk || trap_set_chk, oper, aux::stringid(ID, 3))?;
        if tc.is_location(duel, LOCATION_HAND) {
            duel.shuffle_hand(tp)?;
        }
        if !success && success_chk {
            success = true;
        }
    }
    duel.special_summon_complete()?;
    let c = e.get_handler(duel);
    if success && c.is_relate_to_effect(duel, e) && e.get_chain_data(duel, NUMBER_OF_TARGETS) == Some(2) {
        duel.break_effect()?;
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
