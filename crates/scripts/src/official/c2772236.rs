// Wattchimera
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 2772236;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_WATT], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_THUNDER, Some(sc), sumtype, tp)
    }
}

// Copies on the field to attack directly.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && duel.get_attack_target().is_none()
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, 1 - a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.ep, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, 1 - a.tp, 1);
    duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    Ok(())
}
