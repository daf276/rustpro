// Ghost Belle & Haunted Mansion
use crate::prelude::*;

const ID: u32 = 73642296;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
}

fn check(duel: &Duel, ev: u8, category: u64) -> bool {
    let info = duel.get_operation_info(ev, category);
    let possible = duel.get_possible_operation_info(ev, category);
    if info.is_none() && possible.is_none() {
        return false;
    }
    let mut g = CardSet::default();
    let mut loc = 0;
    for info in [info, possible].into_iter().flatten() {
        if let Some(cards) = info.op_cards {
            g.merge(cards);
        }
        loc |= info.op_param as u32;
    }
    let in_grave =
        |c: CardId, duel: &Duel| c.is_location(duel, LOCATION_GRAVE) && (category != CATEGORY_SPECIAL_SUMMON || c.is_monster(duel));
    loc & LOCATION_GRAVE > 0 || (!g.is_empty() && g.is_exists(duel, in_grave, 1, None))
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    if !duel.is_chain_negatable(ev) {
        return false;
    }
    [
        CATEGORY_SPECIAL_SUMMON,
        CATEGORY_REMOVE,
        CATEGORY_TOHAND,
        CATEGORY_TODECK,
        CATEGORY_TOEXTRA,
    ]
    .into_iter()
    .any(|category| check(duel, ev, category))
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}
