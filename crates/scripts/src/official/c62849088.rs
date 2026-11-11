// The Iris Swordsoul
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 62849088;

// A copy on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];
// The effects whose condition registers a flag while a chain resolves; a summon from the Deck is rarer.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_SUMMON | TIMING_SPSUMMON);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_label(duel, 1);
    e2.set_condition_mut(duel, hdexcon);
    e2.set_target(duel, hdextg);
    e2.set_operation(duel, hdexop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_label(duel, 2);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_code(duel, EVENT_CUSTOM + ID);
    e4.set_label(duel, 3);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_operation(duel, regop);
    c.register_effect(duel, e5);
}

// The group e4 and e5 share as their label object, created with the card's effects.
fn group_key(duel: &Duel, e: EffectId) -> u64 {
    u64::from(ID) << 16 | e.get_owner(duel).index() as u64
}

fn label_group(duel: &Duel, e: EffectId) -> CardSet {
    duel.get_script_group(group_key(duel, e)).unwrap_or_default()
}

fn spcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_disabled(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_main_phase() && duel.is_existing_matching_card(spcfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_HAND as i32);
    Ok(false)
}

fn hdexfilter(c: CardId, duel: &Duel, tp: u8, loc: u32) -> bool {
    c.is_original_type(duel, TYPE_MONSTER)
        && c.is_summon_player(duel, 1 - tp)
        && c.is_summon_location(duel, loc)
        && (loc != LOCATION_EXTRA || c.is_on_field(duel))
}

fn hdexcon(duel: &mut Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let label = e.get_label(duel) as u32;
    if c.get_flag_effect(duel, ID + label) > 0 {
        return false;
    }
    let (loc, eg) = match label {
        1 => (LOCATION_HAND, a.eg),
        2 => (LOCATION_DECK, a.eg),
        3 => (LOCATION_EXTRA, Some(label_group(duel, e))),
        _ => (0, a.eg),
    };
    if eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| hdexfilter(c, duel, tp, loc), 1, None)) {
        if duel.get_current_chain() > 0 {
            c.register_flag_effect(duel, ID + label, RESET_CHAIN, 0, 1);
        }
        return true;
    }
    false
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn hdextg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let label = e.get_label(duel);
    let extra = |duel: &Duel| label_group(duel, e).filter(duel, |c, duel| hdexfilter(c, duel, tp, LOCATION_EXTRA), None);
    if chk == 0 {
        return Ok(match label {
            1 => {
                duel.get_location_count(tp, LOCATION_MZONE) > 0
                    && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
            }
            2 => duel.is_player_can_draw(tp, 2),
            3 => !extra(duel).is_empty(),
            _ => false,
        });
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    match label {
        1 => duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32),
        2 => {
            duel.set_target_player(tp);
            duel.set_target_param(2);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
        }
        3 => {
            let g = extra(duel);
            duel.set_target_card(g);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
        }
        _ => {}
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(false)
}

fn hdexop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 if duel.get_location_count(tp, LOCATION_MZONE) > 0 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        2 => {
            let p = duel.get_chain_info::<u8>(0, CHAININFO_TARGET_PLAYER);
            let d = duel.get_chain_info::<u32>(0, CHAININFO_TARGET_PARAM);
            duel.draw(p, d, REASON_EFFECT)?;
        }
        3 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = label_group(duel, e).filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
            duel.set_script_group(group_key(duel, e), Some(g));
            let g = g.filter_select(duel, tp, |c, duel| hdexfilter(c, duel, tp, LOCATION_EXTRA), 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_flag_effect(tp, ID) > 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let tg =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| hdexfilter(c, duel, tp, LOCATION_EXTRA), None);
    if !tg.is_empty() && !tg.contains(c) {
        for tc in tg {
            tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
        }
        let g = if duel.get_current_chain() == 0 {
            CardSet::default()
        } else {
            label_group(duel, e)
        };
        let mut g = g.filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
        g.merge(tg);
        duel.set_script_group(group_key(duel, e), Some(g));
        duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}
