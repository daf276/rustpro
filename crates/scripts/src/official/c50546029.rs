// G Golem Dignified Trilithon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 50546029;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(lc), sumtype, tp)
    }
}

// On the field and Link Summoned from EARTH Normal Monsters (Doron, Flame Viper) with more in the hand to discard, and Mage Power and
// Mind Control from the opponent, whose targeting is negated, with monsters to attack into it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (756652, MZONE),
    (756652, HAND),
    (2830619, HAND),
    (6103114, MZONE | OPPONENT),
    (6103114, MZONE | OPPONENT),
    (6103114, MZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_MUST_ATTACK);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e2.set_value(duel, Value::Card(|duel, e, c| i32::from(c == e.get_handler(duel))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_CONFIRM);
    e3.set_condition_mut(duel, atkcon);
    e3.set_cost(duel, atkcost);
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, discon);
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn atkcon(duel: &mut Duel, a: &Args) -> bool {
    let (_, bc) = duel.get_battle_monster(a.tp);
    let Some(bc) = bc else { return false };
    a.e.set_label_object_group(duel, Some(bc.into()));
    true
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_able_to_grave_as_cost(duel)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cfilter, LOCATION_HAND)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let d = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, d, 1, 1 - tp, -200);
    duel.set_operation_info(0, CATEGORY_DISABLE, d, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let d = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if let Some(d) = d
        && d.is_relate_to_battle(duel)
        && d.is_faceup(duel)
        && d.is_controler(duel, 1 - tp)
    {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -200);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        d.register_effect(duel, e1);
        duel.negate_related_chain(d, RESET_TURN_SET);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        d.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_DISABLE_EFFECT);
        e3.set_value(duel, RESET_TURN_SET as i32);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        d.register_effect(duel, e3);
    }
    Ok(())
}

fn tfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_location(duel, LOCATION_MZONE) && c.is_type(duel, TYPE_LINK) && c.is_controler(duel, tp)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return false };
    if a.rp == tp || a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(ev, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| tfilter(c, duel, tp), 1, None)) && duel.is_chain_disablable(ev)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    if chk == 0 {
        return Ok(!rc.is_status(duel, STATUS_DISABLED));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_effect(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
