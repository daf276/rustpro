// Ash Blossom & Joyous Spring
use crate::prelude::*;

const ID: u32 = 14558127;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
}

// s.check(ev,re) returns a closure over ev and re; here they are parameters.
fn check(duel: &Duel, ev: u8, re: EffectId, category: u64, checkloc: bool) -> bool {
    if !checkloc && re.is_has_category(duel, category) {
        return true;
    }
    let info = duel.get_operation_info(ev, category);
    let possible = duel.get_possible_operation_info(ev, category);
    if info.is_none() && possible.is_none() {
        return false;
    }
    if category == CATEGORY_DRAW || category == CATEGORY_DECKDES {
        return true;
    }
    let mut g = CardSet::default();
    let mut dv = 0;
    for info in [info, possible].into_iter().flatten() {
        if let Some(cards) = info.op_cards {
            g.merge(cards);
        }
        dv |= info.op_param as u32;
    }
    dv & LOCATION_DECK != 0 || (!g.is_empty() && g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None))
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let Some(re) = a.re else { return false };
    if re.get_handler(duel).is_disabled(duel) || !duel.is_chain_disablable(ev) {
        return false;
    }
    let check = |category, checkloc| check(duel, ev, re, category, checkloc);
    check(CATEGORY_TOHAND, true)
        || check(CATEGORY_SPECIAL_SUMMON, true)
        || check(CATEGORY_TOGRAVE, true)
        || check(CATEGORY_DRAW, true)
        || check(CATEGORY_DRAW, false)
        || check(CATEGORY_SEARCH, false)
        || check(CATEGORY_DECKDES, true)
        || check(CATEGORY_DECKDES, false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.re.is_some_and(|re| !re.get_handler(duel).is_status(duel, STATUS_DISABLED)));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_effect(a.ev as u8);
    Ok(())
}
