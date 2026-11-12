// Jurrac Velphito
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 65961683;

// In the Extra Deck, Synchro Summoned with Jurrac Megalo (a Level 1 Tuner) and Uraby (a Level 4 Dinosaur), with the opponent's
// Key Maces (often face-down) to battle, and copies on the field to attack them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (93170499, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1784619, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_race_full(duel, RACE_DINOSAUR, Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, valop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn valop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_synchro_summoned(duel) {
        return Ok(());
    }
    let atk: i32 = c.get_material(duel).iter().map(|tc| tc.get_text_attack(duel).max(0)).sum();
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_value(duel, atk);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e2);
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let d = duel.get_attack_target();
    Some(a.e.get_handler(duel)) == duel.get_attacker() && d.is_some_and(|d| d.is_position(duel, POS_FACEDOWN_DEFENSE))
}

fn destg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let d = duel.get_attack_target();
    if chk == 0 {
        return Ok(d.is_some_and(|d| d.is_destructable(duel)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, d.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, _: &Args) -> Script<()> {
    let d = duel.get_attack_target();
    unported!(duel, d.is_none());
    if let Some(d) = d
        && d.is_relate_to_battle(duel)
    {
        duel.destroy(d, REASON_EFFECT)?;
    }
    Ok(())
}
