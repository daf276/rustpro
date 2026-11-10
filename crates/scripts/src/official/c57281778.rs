// Ryu Kokki
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 57281778;

// On the field against Millennium Shield, which survives its attack in Defense Position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (32012841, MZONE | OPPONENT),
    (32012841, MZONE | OPPONENT),
    (32012842, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition_mut(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let t = if a.ev == 1 { duel.get_attacker() } else { duel.get_attack_target() };
    a.e.set_label_object_group(duel, t.map(CardSet::from));
    t.is_some_and(|t| t.is_race(duel, RACE_SPELLCASTER | RACE_WARRIOR))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_relate_to_battle(duel)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.is_relate_to_battle(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
