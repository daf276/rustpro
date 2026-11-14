// Schuberta the Melodious Maestra
use super::c56208713::Materials;
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 57594700;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS];
pub const MATERIAL_SETCODE: &[u16] = &[SET_MELODIOUS];

// In the Extra Deck, Fusion Summoned by Polymerization from Aria and Elegy, with cards in both GYs to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (40502912, MZONE),
    (79514956, MZONE),
    (40502912, GRAVE),
    (79514956, GRAVE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_NO_TURN_RESET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 3, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel.get_target_cards(e).unwrap_or_default();
    let ct = duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    let c = e.get_handler(duel);
    if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_value(duel, ct * 200);
        c.register_effect(duel, e1);
    }
    Ok(())
}
