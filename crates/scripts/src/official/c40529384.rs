// Exploder Dragonwing
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 40529384;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_DRAGON, Some(sc), sumtype, tp))
    }
}

// In the Extra Deck, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Luster Dragon #2 (Level 6 Dragon), against the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16449363, MZONE),
    (16449363, MZONE),
    (17658803, MZONE),
    (17658803, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn battle_opponent(duel: &Duel, c: CardId) -> Option<CardId> {
    let tc = duel.get_attacker();
    if tc == Some(c) { duel.get_attack_target() } else { tc }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let tc = battle_opponent(duel, c);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_faceup(duel) && tc.get_attack(duel) <= c.get_attack(duel)));
    }
    unported!(duel, tc.is_none());
    if let Some(tc) = tc {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, 0, 0);
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, tc.get_attack(duel));
    }
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let tc = battle_opponent(duel, c);
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_battle(duel)
        && tc.get_attack(duel) <= c.get_attack(duel)
    {
        let atk = tc.get_attack(duel);
        if duel.destroy(tc, REASON_EFFECT)? != 0 {
            duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
