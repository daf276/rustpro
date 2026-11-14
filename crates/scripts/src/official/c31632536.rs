// Laval Dual Slasher
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 31632536;

pub const LISTED_SERIES: &[u16] = &[SET_LAVAL];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(sc), sumtype, tp)
    }
}

// Laval Coatl (Level 2 Tuner) with Flame Manipulator for a Synchro Summon from the Extra Deck, Laval monsters in the Graveyard and
// Level 4 monsters on the opponent's side to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (45439263, MZONE),
    (34460851, MZONE),
    (2407147, GRAVE),
    (15169262, GRAVE),
    (8041569, GRAVE),
    (45042329, MZONE | OPPONENT),
    (45042329, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_operation(duel, caop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_operation(duel, caop2);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_PIERCE);
    e3.set_condition(duel, pcon);
    c.register_effect(duel, e3);
}

fn caop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let defense_target = duel.get_attack_target().is_some_and(|d| d.is_defense_pos(duel));
    e.set_label(duel, i64::from(Some(e.get_handler(duel)) == duel.get_attacker() && defense_target));
    Ok(())
}

fn laval_in_grave(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(|c, duel| c.is_set_card(duel, &[SET_LAVAL]), tp, LOCATION_GRAVE, 0, None)
}

fn caop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if e.get_label_object(duel).is_some_and(|e1| e1.get_label(duel) == 1)
        && c.is_relate_to_battle(duel)
        && c.can_chain_attack(duel)
        && laval_in_grave(duel, tp) >= 2
    {
        duel.chain_attack();
    }
    Ok(())
}

fn pcon(duel: &Duel, a: &Args) -> bool {
    laval_in_grave(duel, a.e.get_handler(duel).get_controler(duel)) >= 3
}
