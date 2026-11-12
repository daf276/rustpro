// Gouki The Giant Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 47946130;

// Link Summoned from three Gouki monsters (Gouki Guts and Gouki Machine Suprex), with Mage Power's targeting to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, MZONE),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<3, 0>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_NEGATE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, negcon);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.get_attack(duel) != c.get_base_attack(duel)
    });
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.get_owner(duel) != e.get_handler(duel)
        && te.is_monster_effect(duel)
        && e.get_handler_player(duel) != te.get_handler_player(duel)
        && te.get_owner(duel).get_attack(duel) <= e.get_handler(duel).get_attack(duel)
        && te.is_activated(duel)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp, ev) = (a.e.get_handler(duel), a.tp, a.ev as u8);
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    if a.rp == tp || !a.re.is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) {
        return false;
    }
    let mut lg = c.get_linked_group(duel);
    lg.insert(c);
    let tg: Option<CardSet> = duel.get_chain_info(ev, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| lg.is_exists(duel, |c, _| tg.is_contains(c), 1, None)) && duel.is_chain_negatable(ev)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(re) = a.re else { return Ok(()) };
    if !c.is_faceup(duel)
        || c.get_attack(duel) < 500
        || !c.is_relate_to_effect(duel, e)
        || u32::from(duel.get_current_chain()) != a.ev + 1
        || c.is_status(duel, STATUS_BATTLE_DESTROYED)
    {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, -500);
    c.register_effect(duel, e1);
    if !c.is_immune_to_effect(duel, e1)
        && c.is_has_effect(duel, EFFECT_REVERSE_UPDATE).is_none()
        && duel.negate_activation(a.ev as u8)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 1000);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}
