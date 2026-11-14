// Dyna, Hero Fur Hire
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 25123713;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
// Revived by Mayhem Fur Hire beside other Fur Hire monsters, with cards in the opponent's Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (91405870, HAND),
    (91405870, HAND),
    (93850652, MZONE),
    (47647354, MZONE),
    (26202165, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atlimit));
    c.register_effect(duel, e2);
}

fn fur_hire_faceup(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE])
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(fur_hire_faceup, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let cg = duel.get_matching_group(fur_hire_faceup, tp, LOCATION_MZONE, 0, None);
    let ct = cg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, 0, LOCATION_GRAVE, 1, ct as u8, None)?;
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn atlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE]) && c != e.get_handler(duel))
}
