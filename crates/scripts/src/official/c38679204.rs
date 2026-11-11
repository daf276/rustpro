// Vylon Stella
use super::c1281505::{add_equip_trigger, eqtg};
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 38679204;

// Two copies and Millennium Shields (DEF 3000) on the field, with Elfnote Lucinas on the opponent side to attack, so a Stella
// is destroyed and equipped to a Shield, which survives the attack that Stella then answers by destroying the attacker.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (32012841, MZONE),
    (32012841, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_equip_trigger(duel, c, ID, eqtg);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition_mut(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let ec = a.e.get_handler(duel).get_equip_target(duel);
    let (attacker, target) = (duel.get_attacker(), duel.get_attack_target());
    let dt = if ec == attacker {
        target
    } else if ec == target {
        attacker
    } else {
        None
    };
    a.e.set_label_object_group(duel, dt.map(CardSet::from));
    dt.is_some_and(|dt| dt.is_relate_to_battle(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let dt = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, dt, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(dt) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && dt.is_relate_to_battle(duel)
    {
        duel.destroy(dt, REASON_EFFECT)?;
    }
    Ok(())
}
