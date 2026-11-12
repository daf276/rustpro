// Battlin' Boxer Rabbit Puncher
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 61679541;

// On the field attacking the opponent's Defense Position monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (14558127, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_target(duel, targ);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn targ(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let d = duel.get_attack_target();
    if chk == 0 {
        return Ok(
            duel.get_attacker() == Some(a.e.get_handler(duel)) && d.is_some_and(|d| d.is_defense_pos(duel) && d.is_relate_to_battle(duel))
        );
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, d.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn op(duel: &mut Duel, _: &Args) -> Script<()> {
    if let Some(d) = duel.get_attack_target()
        && d.is_relate_to_battle(duel)
        && d.is_defense_pos(duel)
    {
        duel.destroy(d, REASON_EFFECT)?;
    }
    Ok(())
}
