// Swordsoul Supreme Sovereign - Chengying
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 96633955;

// In the Extra Deck and on the field, Synchro Summoned with Jiaotu and Phantasm Spiral Dragon, with Pot of Desires to banish,
// the opponent's Dark Hole and Raigeki to destroy it and cards in the opponent's field and Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (25935625, MZONE),
    (56649609, MZONE),
    (35261759, HAND),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (56649609, MZONE | OPPONENT),
    (61488417, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(
        duel,
        Value::Card(|duel, _, _| duel.get_field_group_count(0, LOCATION_REMOVED, LOCATION_REMOVED) as i32 * 100),
    );
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_value(
        duel,
        Value::Card(|duel, _, _| duel.get_field_group_count(0, LOCATION_REMOVED, LOCATION_REMOVED) as i32 * -100),
    );
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_code(duel, EFFECT_DESTROY_REPLACE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target(duel, desreptg);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_REMOVE);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e6.set_property(duel, EFFECT_FLAG_DELAY);
    e6.set_code(duel, EVENT_REMOVE);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_count_limit(duel, 1, ID);
    e6.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| !c.is_type(duel, TYPE_TOKEN), 1, None)
    });
    e6.set_target(duel, remtg);
    e6.set_operation(duel, remop);
    c.register_effect(duel, e6);
}

fn repfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && c.is_reason(duel, REASON_EFFECT)
            && duel.is_existing_matching_card(repfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(tp, repfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(true)
}

fn remtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let player = a.e.get_handler_player(duel);
        return Ok(duel.is_player_affected_by_effect(player, CARD_SPIRIT_ELIMINATION).is_none()
            && duel.is_existing_matching_card(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_ONFIELD, 1, None)
            && duel.is_existing_matching_card(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 2, 0, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn remop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_matching_group(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_ONFIELD, None);
    let g2 = duel.get_matching_group(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_GRAVE, None);
    if !g1.is_empty() && !g2.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let mut sg1 = g1.select(duel, tp, 1, 1, None)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg2 = g2.select(duel, tp, 1, 1, None)?;
        sg1.merge(sg2);
        duel.hint_selection(sg1);
        duel.remove(sg1, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
