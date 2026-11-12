// Hyper Psychic Blaster
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 95526884;

// In the Extra Deck, Synchro Summoned with Stinger the Electric Expeller (Level 3 Psychic Tuner) and two Chosen by the World Chalice (Level 3 Psychic), on the field to attack the opponent's Defense Position monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (57649113, MZONE),
    (ID, MZONE),
    (22916281, MZONE),
    (22916281, MZONE),
    (10071456, MZONE | OPPONENT),
    (10071456, MZONE | OPPONENT),
    (10071456, MZONE | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_condition_mut(duel, reccon);
    e2.set_target(duel, rectg);
    e2.set_operation(duel, recop);
    c.register_effect(duel, e2);
}

fn reccon(duel: &mut Duel, a: &Args) -> bool {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let (Some(attacker), Some(d)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    let def = if d.is_location(duel, LOCATION_MZONE) {
        d.get_defense(duel)
    } else {
        d.get_previous_defense_on_field(duel)
    };
    let m = attacker.get_attack(duel) - def;
    e.set_label(duel, i64::from(m));
    c == attacker && def >= 0 && m > 0 && d.get_battle_position(duel) & POS_DEFENSE != 0
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (tp, label) = (a.tp, a.e.get_label(duel));
    duel.set_target_player(tp);
    duel.set_target_param(label as u32);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, label as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
