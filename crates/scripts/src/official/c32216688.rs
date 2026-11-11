// R.B. The Brute Blues
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 32216688;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_MACHINE, Some(lc), sumtype, tp)
    }
}

// On the field with R.B. Spells in the Deck to search; the effects needing a linked R.B. monster depend on the random zones.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (5109321, DECK), (78710386, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, rbcon);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_condition(duel, rbcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE);
    e3a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_condition(duel, rbcon);
    e3a.set_value(duel, 1);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3b);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn rbcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_RB]));
    a.e.get_handler(duel).get_linked_group(duel).is_exists(duel, filter, 1, None)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_RB]) && c.get_base_attack(duel) >= 0
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    c.get_linked_group(duel)
        .filter(duel, atkfilter, None)
        .get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as i32
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RB]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
