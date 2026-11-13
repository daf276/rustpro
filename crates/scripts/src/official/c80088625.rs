// Binary Blader
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 80088625;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_NORMAL, Some(lc), sumtype, tp)
    }
}

// Copies on both sides in a row (Left and Right arrows, so mutually linked), with Millennium Shields (DEF 3000) to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (32012841, MZONE | OPPONENT),
    (32012841, MZONE | OPPONENT),
    (32012841, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atkcon);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_condition_mut(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_mutual_linked_group_count(duel) >= 1
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let d = duel.get_attack_target();
    if chk == 0 {
        return Ok(d.is_some_and(|d| d.is_controler(duel, 1 - a.tp)));
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_target_range(duel, 0, LOCATION_MZONE);
        e2.set_target_card(duel, indestg);
        e2.set_value(duel, 1);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn indestg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    Some(c) == e.get_handler(duel).get_battle_target(duel)
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    a.e.set_label_object_group(duel, bc.map(Into::into));
    c.get_mutual_linked_group_count(duel) >= 2
        && bc.is_some_and(|bc| bc.is_status(duel, STATUS_OPPO_BATTLE) && bc.is_relate_to_battle(duel))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let bc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| bc.is_able_to_remove(duel)));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_REMOVE, bc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(bc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if bc.is_relate_to_battle(duel) && bc.is_controler(duel, 1 - tp) {
        duel.remove(bc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
