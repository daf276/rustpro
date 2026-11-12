// Flower Cardian Lightshower
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 42291297;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 3;
    const MAX2: usize = 3;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Three in the Extra Deck, Synchro Summoned with Melffy Pinny (a Level 2 Tuner) and three Melffy Rabby (Level 2); the opponent's
// draw triggers the damage and their end phase the choice.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (34800281, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(0),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_FLOWER_CARDIAN]));
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_DRAW);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, |_, a| a.ep != a.tp && a.r == REASON_RULE);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, stock::damage);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_category(duel, CATEGORY_DISABLE);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_PHASE + PHASE_END);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e5.set_target(duel, eptg);
    e5.set_operation(duel, epop);
    c.register_effect(duel, e5);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_target_param(1500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1500);
    Ok(false)
}

fn eptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EFFECT);
    let op = if c.is_negatable_monster(duel) {
        duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 1)])?
    };
    if op == 0 {
        e.set_category(duel, 0);
    } else {
        e.set_category(duel, CATEGORY_DISABLE);
        duel.set_operation_info(0, CATEGORY_DISABLE, Some(c.into()), 1, 0, 0);
    }
    e.set_label(duel, i64::from(op));
    Ok(false)
}

fn epop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if e.get_label(duel) == 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_SKIP_DP);
        e1.set_target_range(duel, 1, 0);
        e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN);
        duel.register_effect(e1, tp);
    } else if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        duel.negate_related_chain(c, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_OPPO_TURN);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        c.register_effect(duel, e2);
    }
    Ok(())
}
