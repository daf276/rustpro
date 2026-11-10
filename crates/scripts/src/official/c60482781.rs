// Mystic Swordsman LV6
use super::c47507260::{descon, destg};
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 60482781;

// On the field with face-down monsters (Tomozaurus) on the opponent's side to attack and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_LIMIT_SUMMON_PROC);
    e2.set_condition(duel, |_, a| a.c.is_none());
    c.register_effect(duel, e2);
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let d = duel.get_attack_target();
    // The Lua indexes a nil target here and errors.
    unported!(duel, d.is_none());
    if let Some(d) = d
        && d.is_relate_to_battle(duel)
    {
        if d.get_owner(duel) != tp && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.destroy_full(d, REASON_EFFECT, LOCATION_DECK, duel.get_reason_player())?;
        } else {
            duel.destroy(d, REASON_EFFECT)?;
        }
    }
    Ok(())
}
