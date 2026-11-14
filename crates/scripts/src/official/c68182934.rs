// Sky Dragoons of Draconia
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, PZONE};

const ID: u32 = 68182934;

// In the Pendulum Zone, with Normal Monsters (Summoned Skull) to deal battle damage to the opponent's Sangans' controller.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (70781052, MZONE),
    (70781052, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.and_then(|eg| eg.get_first())
        .is_some_and(|tc| a.ep != tp && tc.is_controler(duel, tp) && tc.is_type(duel, TYPE_NORMAL))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
