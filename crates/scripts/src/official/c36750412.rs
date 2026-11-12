// Magna Drago
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 36750412;

// Copies in the hand to Normal Summon face-up and on the field, attacking or attacked by weak monsters (Doron, Kagemusha of the Blue
// Flame) of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (756652, MZONE | OPPONENT),
    (756652, MZONE | OPPONENT),
    (15401633, MZONE | OPPONENT),
    (15401633, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, |_, a| a.ep != a.tp);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_value(duel, 200);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
