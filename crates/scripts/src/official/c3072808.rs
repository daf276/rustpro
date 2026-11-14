// Blackwing - Hillen the Tengu-wind
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 3072808;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// In the Graveyard with Gale, and Blue-Eyes White Dragons of the opponent's to attack directly for 2000 or more.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (2009101, GRAVE),
    (2009101, GRAVE),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep == a.tp && a.ev >= 2000 && duel.get_attack_target().is_none()
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 3) && c.is_set_card(duel, &[SET_BLACKWING]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g = duel
        .select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 2, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    if !c.is_relate_to_effect(duel, e) || !c.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
        return Ok(());
    }
    if !tc.is_relate_to_effect(duel, e) || !tc.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    tc.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    tc.register_effect(duel, e4);
    duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)?;
    duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    duel.special_summon_complete()?;
    Ok(())
}
