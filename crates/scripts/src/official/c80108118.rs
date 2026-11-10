// X-Saber Urbellum
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 80108118;

// Two on the field, and two in the Extra Deck Synchro Summoned with X-Saber Airbellum (Level 3 Tuner) and X-Saber Anu Piranha (Level 4).
// Raigeki clears the opponent for a direct attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (90508760, MZONE),
    (23115241, MZONE),
    (12580477, HAND),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && duel.get_field_group_count(a.tp, 0, LOCATION_HAND) >= 4
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
