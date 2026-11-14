// Icejade Creation Aegirocassis
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 1295442;

pub const LISTED_SERIES: &[u16] = &[SET_ICEJADE];
// In the hand and the Graveyard, with Icejade Ran Aegirine on the field to equip it to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (18494511, MZONE), (18494511, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_EQUIP);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e3.set_target(duel, eqtg);
    e3.set_operation(duel, eqop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    e4.set_target(duel, stock::special_summon_self_target_player_0);
    e4.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(
        aux::faceup_filter(CardId::is_monster),
        e.get_handler_player(duel),
        LOCATION_REMOVED,
        LOCATION_REMOVED,
        None,
    ) as i32
        * 400
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ICEJADE])
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(eqfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    }
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_SZONE) > 0
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_controler(duel, tp)
        && duel.equip(tp, c, tc)?
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(
            duel,
            Value::Card(|duel, e, c| i32::from(Some(c) == e.get_label_object_group(duel).and_then(|g| g.get_first()))),
        );
        e1.set_label_object_group(duel, Some(tc.into()));
        c.register_effect(duel, e1);
    } else if c.is_location(duel, LOCATION_HAND) {
        duel.send_to_grave(c, REASON_RULE)?;
    }
    Ok(())
}
