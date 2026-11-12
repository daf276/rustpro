// Lightning Warrior
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 87259077;

// In the Extra Deck, Synchro Summoned with Naturia Tulip (a Level 2 Tuner) and Zanki (Level 5), and on the field, to attack the
// opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (70261145, MZONE),
    (70261145, MZONE),
    (30090452, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_condition(duel, damcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let t = if a.ev == 1 { duel.get_attacker() } else { duel.get_attack_target() };
    if !c.is_relate_to_battle(duel) || c.is_facedown(duel) {
        return false;
    }
    t.is_some_and(|t| t.is_location(duel, LOCATION_GRAVE) && t.is_monster(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 0);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let hand = duel.get_field_group_count(p, LOCATION_HAND, 0);
    duel.damage(p, hand as u32 * 300, REASON_EFFECT)?;
    Ok(())
}
