// Goblin Biker Grand Bash
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 64327901;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];
// Goblin monsters of different Levels on the field for the Level change, and in the hand for the extra Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (24079759, MZONE),
    (24079759, MZONE),
    (63665875, MZONE),
    (12057781, MZONE),
    (78658564, HAND),
    (78658564, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GOBLIN]));
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GOBLIN]));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_LVCHANGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, lvtg);
    e4.set_operation(duel, lvop);
    c.register_effect(duel, e4);
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GOBLIN]));
    300 * duel.get_matching_group_count(filter, e.get_handler_player(duel), LOCATION_MZONE, 0, None) as i32
}

fn lvfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GOBLIN]) && c.has_level(duel) && c.is_can_be_effect_target(duel, e)
}

fn level_classes(duel: &Duel, sg: CardSet) -> usize {
    sg.get_class_count(duel, |c, duel| i64::from(c.get_level(duel)))
}

fn original_level_sum_unused(duel: &Duel, sg: CardSet) -> bool {
    let sum = sg.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel)));
    !sg.is_exists(duel, |c, duel| i64::from(c.get_level(duel)) == sum, 1, None)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (level_classes(duel, sg) == 2 || original_level_sum_unused(duel, sg), false)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| lvfilter(c, duel, e), tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TARGET)?;
    duel.set_target_card(sg);
    let b1 = level_classes(duel, sg) == 2;
    let b2 = original_level_sum_unused(duel, sg);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.unwrap_or(0) as i64);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(sg), 2, tp, 0);
    Ok(false)
}

fn change_level(duel: &mut Duel, c: CardId, tc: CardId, lv: i32) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_LEVEL);
    e1.set_value(duel, lv);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if g.len() != 2 {
        return Ok(());
    }
    let op = e.get_label(duel);
    let c = e.get_handler(duel);
    if op == 1 {
        if level_classes(duel, g) == 1 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 4));
        let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        duel.hint_selection_full(tc, true);
        let lv = g.difference(tc.into()).get_first().map_or(0, |oc| oc.get_level(duel));
        change_level(duel, c, tc, lv as i32);
    } else if op == 2 {
        let lv = g.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel)));
        if g.is_exists(duel, |c, duel| i64::from(c.get_level(duel)) == lv, 1, None) {
            return Ok(());
        }
        for tc in g {
            change_level(duel, c, tc, lv as i32);
        }
    }
    Ok(())
}
