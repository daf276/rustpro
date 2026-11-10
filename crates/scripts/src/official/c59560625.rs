// Shift
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 59560625;

// Kojikocy (Normal Monsters) on the field for Elfnote Lucina to attack and to take the target of Mage Power, an Equip Spell.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (13597785, MZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_condition(duel, condition1);
    e1.set_target(duel, target1);
    e1.set_operation(duel, activate1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_condition_mut(duel, condition2);
    e2.set_target(duel, target2);
    e2.set_operation(duel, activate2);
    c.register_effect(duel, e2);
}

fn condition1(duel: &Duel, a: &Args) -> bool {
    let (Some(attacker), Some(target)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    attacker.is_controler(duel, 1 - a.tp) && target.is_controler(duel, a.tp)
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let Some(attacker) = duel.get_attacker() else { return Ok(false) };
    let (ag, _) = attacker.get_attackable_target(duel);
    let at = duel.get_attack_target().map(CardSet::from);
    let filter = |c: CardId, duel: &Duel| c.is_can_be_effect_target(duel, e);
    if chk == 0 {
        return Ok(ag.is_exists(duel, filter, 1, at));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tg = ag.filter_select(duel, tp, filter, 1, 1, at)?;
    duel.set_target_card(tg);
    Ok(false)
}

fn activate1(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && let Some(attacker) = duel.get_attacker()
        && tc.is_relate_to_effect(duel, a.e)
        && !attacker.is_immune_to_effect(duel, a.e)
    {
        duel.change_attack_target(Some(tc), false);
    }
    Ok(())
}

fn condition2(duel: &mut Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let Some(re) = a.re else { return false };
    if a.rp == tp || !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS) else {
        return false;
    };
    let Some(tc) = g.get_first().filter(|_| g.len() == 1) else {
        return false;
    };
    e.set_label_object_group(duel, Some(tc.into()));
    tc.is_controler(duel, tp) && tc.is_location(duel, LOCATION_MZONE)
}

fn filter(c: CardId, duel: &Duel, ct: u8) -> bool {
    duel.check_mut(|duel| duel.check_chain_target(ct, c))
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp, ev) = (a.e, a.tp, a.ev as u8);
    let tc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) != tc && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel, ev));
    }
    let exception = tc.map(CardSet::from);
    let filter = |c, duel: &Duel| filter(c, duel, ev);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, exception)?;
    Ok(false)
}

fn activate2(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_target_card(a.ev as u8, tc.into());
    }
    Ok(())
}
