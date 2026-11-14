// X-Saber Souza
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 63612442;

pub const LISTED_SERIES: &[u16] = &[SET_X_SABER];
// In the Extra Deck, Synchro Summoned with X-Saber Airbellum (Level 3 Tuner) and Anu Piranha (Level 4), with a second X-Saber
// to Tribute and weak opponent monsters (Key Mace) to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (90508760, MZONE),
    (23115241, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_set_card(duel, &[SET_X_SABER]))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn xsaber(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_X_SABER])
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, xsaber, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, xsaber, (1, 1), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0 || c.get_flag_effect(duel, ID + 1) == 0);
    }
    let t1 = c.get_flag_effect(duel, ID);
    let t2 = c.get_flag_effect(duel, ID + 1);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let op = if t1 == 0 && t2 == 0 {
        duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
    } else if t1 == 0 {
        duel.select_option(tp, &[aux::stringid(ID, 1)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 2)])?;
        1
    };
    e.set_label(duel, op.into());
    if op == 0 {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    } else {
        c.register_flag_effect(duel, ID + 1, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.e.get_label(duel) == 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_category(duel, CATEGORY_DESTROY);
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
        e1.set_code(duel, EVENT_BATTLE_START);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        e1.set_condition_mut(duel, descon);
        e1.set_target(duel, destg);
        e1.set_operation(duel, desop);
        c.register_effect(duel, e1);
    } else {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3012);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.get_owner(duel).is_trap(duel)));
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let mut d = duel.get_attack_target();
    if d == Some(a.e.get_handler(duel)) {
        d = duel.get_attacker();
    }
    a.e.set_label_object_group(duel, d.map(CardSet::from));
    d.is_some()
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let d = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, d, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let d = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, d.is_none());
    if let Some(d) = d
        && d.is_relate_to_battle(duel)
    {
        duel.destroy(d, REASON_EFFECT)?;
    }
    Ok(())
}
