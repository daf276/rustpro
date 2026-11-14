// Mecha Phantom Beast Concoruda
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 53451824;

pub const LISTED_SERIES: &[u16] = &[SET_MECHA_PHANTOM_BEAST];
// A copy on the field and three in the Extra Deck, Synchro Summoned with Mecha Phantom Beast Warbluran (a Level 1 Tuner, which leaves a
// Token behind) and Frostosaurus (Level 6), with Raigeki in the opponent's hand to destroy it and Warbluran in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (31480215, MZONE),
    (31480215, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (31480215, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_TOKEN));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_target_range(duel, LOCATION_ONFIELD, 0);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_previous_controler(duel, tp)
        && c.get_reason_player(duel) == 1 - tp
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| c.is_type(duel, TYPE_TOKEN), tp, LOCATION_MZONE, 0, None);
    e.set_label(duel, g.len() as i64);
    if chk == 0 {
        return Ok(!g.is_empty() && g.filter_count(duel, CardId::is_releasable, None) == g.len());
    }
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MECHA_PHANTOM_BEAST])
        && c.is_level_below(duel, 4)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 1 - e.get_label(duel) as i32
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
