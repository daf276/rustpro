// Monarchic Perfection
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 59983249;

type Filter = fn(CardId, &Duel) -> bool;

pub const LISTED_SERIES: &[u16] = &[SET_MONARCH];

// Copies on the field, Monarch Spells in the Graveyard to banish, and Monarchs to Tribute Summon with Mystical Sheep #2.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (22842126, GRAVE),
    (22842126, GRAVE),
    (9283801, GRAVE),
    (4929256, HAND),
    (73125233, HAND),
    (51945556, HAND),
    (83464209, MZONE),
    (83464209, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e1.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None))
    });
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn effcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MONARCH]) && c.is_spell_trap(duel) && c.is_able_to_remove_as_cost(duel)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, -100);
    stock::banish_cost(duel, a, chk, effcostfilter, LOCATION_GRAVE)
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let cost_skip = e.get_label(duel) != -100;
    let available = |duel: &Duel, flag: u32| cost_skip || !duel.has_flag_effect(tp, flag);
    let b1 = available(duel, ID) && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    let b2 =
        available(duel, ID + 1) && duel.is_existing_matching_card(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
    let b3 = available(duel, ID + 2) && duel.is_existing_matching_card(discardable, tp, 0, LOCATION_HAND, 1, None);
    let b4 = available(duel, ID + 3)
        && duel.is_existing_matching_card(CardId::is_facedown, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()));
    let b5 =
        available(duel, ID + 4) && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
    let b6 = available(duel, ID + 5)
        && duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(
            duel.is_existing_matching_card(CardId::is_tribute_summoned, tp, LOCATION_MZONE, 0, 1, None)
                && (b1 || b2 || b3 || b4 || b5 || b6),
        );
    }
    let options = [b1, b2, b3, b4, b5, b6]
        .into_iter()
        .zip(1..)
        .map(|(available, n)| (available, aux::stringid(ID, n)))
        .collect::<Vec<_>>();
    let op = duel.select_effect(tp, &options)?.unwrap_or(0);
    e.set_label(duel, op as i64);
    let (category, opinfo): (u64, Option<Filter>) = match op {
        1 => (CATEGORY_DESTROY, Some(aux::TRUE)),
        2 => (CATEGORY_DESTROY, Some(CardId::is_spell_trap)),
        3 => (CATEGORY_HANDES, None),
        4 => (CATEGORY_DESTROY, Some(CardId::is_facedown)),
        5 => (CATEGORY_TODECK, None),
        6 => (CATEGORY_REMOVE, None),
        _ => return Ok(false),
    };
    e.set_category(duel, category);
    if !cost_skip {
        duel.register_flag_effect(tp, ID + op as u32 - 1, RESET_PHASE | PHASE_END, 0, 1);
    }
    match (op, opinfo) {
        (3, _) => duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 1),
        (_, Some(filter)) => {
            let location = if op == 1 { LOCATION_MZONE } else { LOCATION_ONFIELD };
            let g = duel.get_matching_group(filter, tp, location, location, None);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
        }
        _ => duel.set_operation_info(0, category, None, 1, tp, LOCATION_ONFIELD as i32),
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let op = a.e.get_label(duel);
    let (hint, filter, location, max): (u64, Filter, u32, u8) = match op {
        1 => (HINTMSG_DESTROY, aux::TRUE, LOCATION_MZONE, 1),
        2 => (HINTMSG_DESTROY, CardId::is_spell_trap, LOCATION_ONFIELD, 2),
        3 => {
            let hg = duel.get_matching_group(discardable, tp, 0, LOCATION_HAND, None);
            if !hg.is_empty() {
                let g = hg.random_select(duel, tp, 1);
                duel.send_to_grave(g, REASON_DISCARD | REASON_EFFECT)?;
            }
            return Ok(());
        }
        4 => (HINTMSG_DESTROY, CardId::is_facedown, LOCATION_ONFIELD, 1),
        5 => (HINTMSG_TODECK, CardId::is_able_to_deck, LOCATION_ONFIELD, 1),
        6 => (HINTMSG_REMOVE, CardId::is_able_to_remove, LOCATION_ONFIELD, 1),
        _ => return Ok(()),
    };
    duel.hint(HINT_SELECTMSG, tp, hint);
    let g = duel.select_matching_card(tp, filter, tp, location, location, 1, max, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    match op {
        5 => duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?,
        6 => duel.remove(g, POS_FACEUP, REASON_EFFECT)?,
        _ => duel.destroy(g, REASON_EFFECT)?,
    };
    Ok(())
}
