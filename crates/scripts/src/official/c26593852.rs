// Ally of Justice Catastor
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 26593852;

// In the Extra Deck, Synchro Summoned from Water Spirit and Mystical Elf, battling the opponent's LIGHT Mystical Elves.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (487395, MZONE),
    (15025844, MZONE),
    (15025844, MZONE | OPPONENT),
    (15025844, MZONE | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| bc.is_faceup(duel) && bc.is_attribute_except(duel, ATTRIBUTE_DARK)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
