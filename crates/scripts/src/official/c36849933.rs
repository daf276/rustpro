// Megalith Och
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 36849933;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

pub struct Params;

impl RitualParams for Params {
    const FORCEDSELECTION: Option<ForcedSelection> = Some(|duel, e, _, g, _| (g.is_contains(e.get_handler(duel)), false));
}

// On the field, it Ritual Summons the copy in the hand with itself, which then draws.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, Ritual::target::<Params, RITPROC_GREATER>);
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) && c.is_controler(duel, a.tp) {
            Ritual::operation::<Params, RITPROC_GREATER>(duel, a)?;
        }
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? > 0 {
        duel.shuffle_hand(a.tp)?;
        duel.break_effect()?;
        duel.discard_hand(a.tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
