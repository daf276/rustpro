// HTS Psyhemuth
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 15028680;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Copies on the field to battle Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let (at, t) = (duel.get_attacker(), duel.get_attack_target());
    if chk == 0 {
        unported!(duel, at.is_none() && t == Some(c));
        return Ok((t == Some(c) && at.is_some_and(|at| at.is_able_to_remove(duel)))
            || (at == Some(c) && t.is_some_and(|t| t.is_able_to_remove(duel))));
    }
    unported!(duel, at.is_none());
    let g: CardSet = [at, t].into_iter().flatten().filter(|c| c.is_relate_to_battle(duel)).collect();
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let g: CardSet = [duel.get_attacker(), duel.get_attack_target()].into_iter().flatten().collect();
    let rg = g.filter(duel, |c, duel| c.is_relate_to_battle(duel), None);
    duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
