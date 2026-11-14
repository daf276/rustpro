// Unchained Twins - Sarama
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

use super::c26236560::{spcon2, spop2, sptg2};

const ID: u32 = 31588572;

pub const LISTED_SERIES: &[u16] = &[SET_UNCHAINED];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field, with Unchained Twins - Aruha and Rakea in the GY to Set and Rakea in the Deck to summon after being destroyed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26236560, GRAVE),
    (53624265, GRAVE),
    (53624265, MZONE),
    (53624265, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon2::<ID>);
    e2.set_target(duel, sptg2::<ID>);
    e2.set_operation(duel, spop2::<ID>);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !c.is_set_card(duel, &[SET_UNCHAINED]) {
        return false;
    }
    if c.is_monster(duel) && !c.is_code(duel, ID) {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
    } else if c.is_spell_trap(duel) {
        (c.is_field_spell(duel) || duel.get_location_count(tp, LOCATION_SZONE) > 0) && c.is_ssetable(duel)
    } else {
        false
    }
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && setfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| setfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, |c, duel| setfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first()) {
        if tc.is_monster(duel) {
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
        } else if tc.is_spell_trap(duel) {
            duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, LOCATION_GRAVE as i32);
        }
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let mut andifyoudo = false;
    if tc.is_monster(duel) {
        andifyoudo =
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)? > 0;
    } else if tc.is_spell_trap(duel) {
        if tc.is_field_spell(duel)
            && let Some(fc) = duel.get_field_card(tp, LOCATION_FZONE, 0)
        {
            duel.send_to_grave(fc, REASON_RULE)?;
            duel.break_effect()?;
        }
        if tc.is_field_spell(duel) || duel.get_location_count(tp, LOCATION_SZONE) > 0 {
            duel.sset(tp, tc)?;
            andifyoudo = true;
        }
    }
    if andifyoudo {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
