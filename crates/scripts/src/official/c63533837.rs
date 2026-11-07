// Cyberse Quantum Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 63533837;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field and in the Extra Deck, Synchro Summoned with Backup @Ignister and Leotron; a Link Monster (Code Talker, Code Generator
// to Link Summon it) to enable the shields, and monsters of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (30118811, MZONE),
    (47226949, MZONE),
    (53413628, EXTRA),
    (30114823, MZONE),
    (30114823, MZONE),
    (47226949, MZONE | OPPONENT),
    (30114823, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, tgcon);
    e2.set_target_card(duel, |duel, e, c| c != e.get_handler(duel));
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_condition(duel, tgcon);
    e3.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e3);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_type(duel, TYPE_LINK),
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_controler(duel, 1 - a.tp) && tc.is_able_to_hand(duel)));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, tc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let mut tc = duel.get_attacker();
    if tc == Some(c) {
        tc = duel.get_attack_target();
    }
    if let Some(tc) = tc
        && tc.is_relate_to_battle(duel)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    if c.is_relate_to_effect(duel, a.e) && c.can_chain_attack(duel) && duel.get_attacker() == Some(c) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_DAMAGE_STEP_END);
        e1.set_operation(duel, caop);
        e1.set_count_limit(duel, 1, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn caop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_handler(duel).can_chain_attack(duel) {
        duel.chain_attack();
    }
    Ok(())
}
