// Hydro Genex
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 47421985;
const GENEX_CONTROLLER: u32 = 68505803;

pub const MATERIAL: &[u32] = &[GENEX_CONTROLLER];
pub const LISTED_NAMES: &[u32] = &[GENEX_CONTROLLER];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[GENEX_CONTROLLER])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned from Genex Controller (a Level 3 Tuner) and Monsturtle (WATER, Level 3), against the
// opponent's Psychic Kappa to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68505803, MZONE),
    (68505803, MZONE),
    (15820147, MZONE),
    (15820147, MZONE),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_condition_mut(duel, reccon);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
}

fn reccon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let t = if a.ev == 1 { duel.get_attacker() } else { duel.get_attack_target() };
    if !c.is_relate_to_battle(duel) || c.is_facedown(duel) {
        return false;
    }
    let Some(t) = t else { return false };
    a.e.set_label(duel, i64::from(t.get_attack(duel)));
    t.is_location(duel, LOCATION_GRAVE) && t.is_monster(duel)
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let label = a.e.get_label(duel) as u32;
    duel.set_target_player(tp);
    duel.set_target_param(label);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, label as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
