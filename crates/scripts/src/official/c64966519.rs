// Dispatchparazzi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 64966519;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Sangans (Effect Monsters) to Link Summon with and to be targeted, and Elfnote Lucina on the opponent's side to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, cbcon);
    e1.set_operation(duel, cbop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn cbcon(duel: &Duel, a: &Args) -> bool {
    let bt = a.eg.unwrap_or_default().get_first();
    a.r != REASON_REPLACE && bt != Some(a.e.get_handler(duel)) && bt.is_some_and(|bt| bt.is_controler(duel, a.tp))
}

fn cbop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
        && let Some(at) = duel.get_attacker()
        && at.can_attack(duel)
        && !at.is_immune_to_effect(duel, e)
        && !c.is_immune_to_effect(duel, e)
    {
        duel.calculate_damage(at, Some(c))?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_BATTLE)
        && c.get_battle_target(duel).is_some_and(|bt| bt.is_controler(duel, 1 - tp))
        && a.rp == 1 - tp
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, 0, 0);
        let recover = tc.get_base_attack(duel) / 2;
        duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, recover);
    }
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel) {
        if duel.destroy(tc, REASON_EFFECT)? == 0 {
            return Ok(());
        }
        let recover = tc.get_base_attack(duel) / 2;
        duel.recover(a.tp, recover as u32, REASON_EFFECT)?;
    }
    Ok(())
}
