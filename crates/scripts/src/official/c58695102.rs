// Re-Cover
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, OPPONENT};

const ID: u32 = 58695102;

// In the Graveyard, with six Synchro Monsters in the opponent's Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (39964797, EXTRA | OPPONENT),
    (39964797, EXTRA | OPPONENT),
    (39964797, EXTRA | OPPONENT),
    (39964797, EXTRA | OPPONENT),
    (39964797, EXTRA | OPPONENT),
    (39964797, EXTRA | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::pay_lp::<2000>);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_field_group_count(tp, LOCATION_EXTRA, 0) as i64 <= duel.get_field_group_count(tp, 0, LOCATION_EXTRA) as i64 - 5
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
