// Trap of the Poisonous Scorpion
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT, SZONE};

const ID: u32 = 212652;

// Elfnote Lucina in the opponent's monster zone to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - a.tp))
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    if let Some(bc) = duel.get_attacker() {
        e.set_label_object_group(duel, Some(bc.into()));
        bc.create_effect_relation(duel, e);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(bc.into()), 1, tp, 0);
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 300);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(bc) = e.get_label_object_group(duel).and_then(|g| g.get_first())
        && bc.is_relate_to_effect(duel, e)
        && bc.is_monster(duel)
        && duel.destroy(bc, REASON_EFFECT)? > 0
    {
        duel.break_effect()?;
        duel.damage(1 - tp, 300, REASON_EFFECT)?;
    }
    Ok(())
}
