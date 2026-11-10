// Crimson Blader
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 80321197;

// In the Extra Deck, Synchro Summoned with Deep Sea Minstrel (Tuner) and Misairuzame (Level 5), with an opponent's monster to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (71978434, MZONE),
    (71978434, MZONE),
    (33178416, MZONE),
    (33178416, MZONE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_condition(duel, aux::bdogcon);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, sumcon);
    e1.set_target_card(duel, |duel, _, c| c.is_level_above(duel, 5));
    let turn = i64::from(duel.get_turn_count());
    e1.set_label(duel, turn);
    if duel.is_turn_player(tp) {
        e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
    } else {
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN, 2);
    }
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_SUMMON);
    duel.register_effect(e2, tp);
    Ok(())
}

fn sumcon(duel: &Duel, a: &Args) -> bool {
    i64::from(duel.get_turn_count()) != a.e.get_label(duel) && duel.get_turn_player() != a.e.get_owner_player(duel)
}
