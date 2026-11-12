// Catapult Warrior
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 37474917;

// In the Extra Deck, Synchro Summoned with Junk Mail (a Level 1 Tuner) and Kojikocy (Level 4), with Junk Converter to release.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (39520293, MZONE),
    (39520293, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (11069680, MZONE),
    (11069680, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
}

fn junk(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_JUNK])
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, junk, (1, 1), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, junk, (1, 1), false, None, None)?;
    let atk = sg.get_first().map_or(0, |c| c.get_text_attack(duel).max(0));
    a.e.set_label(duel, i64::from(atk));
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (tp, label) = (a.tp, a.e.get_label(duel) as i32);
    duel.set_target_player(1 - tp);
    duel.set_target_param(label as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, label);
    Ok(false)
}
