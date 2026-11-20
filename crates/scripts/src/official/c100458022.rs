// Cielo, the Night Sky of Raise Moon - No More Bets!
use engine::OpTarget;

use super::c100458016::{draw, draw_target};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 100458022;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_RAISE_MOON], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

// On the field for the opponent's Raigeki, in the Extra Deck with Level 7 Raise Moon materials, and The Glitter of Raise Moon
// to Special Summon it with a material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (100458018, MZONE),
    (100458018, MZONE),
    (100458017, MZONE),
    (100458026, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e0.set_value(
        duel,
        Value::Material(|_, _, _, sumtype, _| {
            aux::cannotmatfilter(&[SUMMON_TYPE_FUSION, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK].map(u64::from), sumtype)
        }),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_controler(duel, a.tp) && a.rp == 1 - a.tp
    });
    e1.set_target(duel, draw_target);
    e1.set_operation(duel, draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_EMZONE);
    e2.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_EMZONE);
    e3.set_condition(duel, changecon);
    e3.set_operation(duel, changeop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DRAW);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_EMZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, draw_target);
    e4.set_operation(duel, draw);
    e4.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e4);
}

// s.check(ev, re) applied to a category; `checkloc` skips the category of the activated effect itself.
fn check(duel: &Duel, ev: u8, re: Option<EffectId>, category: u64, checkloc: bool) -> bool {
    if !checkloc && re.is_some_and(|re| re.is_has_category(duel, category)) {
        return true;
    }
    let info = duel.get_operation_info(ev, category);
    let possible = duel.get_possible_operation_info(ev, category);
    if info.is_none() && possible.is_none() {
        return false;
    }
    if category == CATEGORY_DRAW {
        return true;
    }
    let param = |op: Option<OpTarget>| op.map_or(0, |op| op.op_param);
    let mut g = CardSet::default();
    for op in [info, possible].into_iter().flatten() {
        if let Some(cards) = op.op_cards {
            g.merge(cards);
        }
    }
    (param(info) | param(possible)) & LOCATION_DECK as i32 != 0
        || (!g.is_empty() && g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None))
}

fn changecon(duel: &Duel, a: &Args) -> bool {
    if a.ep == a.tp {
        return false;
    }
    let ev = a.ev as u8;
    check(duel, ev, a.re, CATEGORY_TOHAND, true)
        || check(duel, ev, a.re, CATEGORY_DRAW, true)
        || check(duel, ev, a.re, CATEGORY_DRAW, false)
        || check(duel, ev, a.re, CATEGORY_SEARCH, false)
}

fn changeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if duel.is_player_can_draw(1 - tp, 1) && duel.select_effect_yes_no_full(tp, c, aux::stringid(ID, 2))? {
        duel.hint(HINT_CARD, 0, ID.into());
        let ev = a.ev as u8;
        duel.change_target_card(ev, CardSet::default());
        duel.change_chain_operation(ev, draw);
    }
    Ok(())
}
