// Uria, Lord of Searing Flames
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, SZONE};

const ID: u32 = 6007213;
const CARD_TRAP_EXTRA_COST: u32 = 16317140;

// Copies in the hand, Special Summoned by sending three face-up Continuous Traps (Abyss of the Sacred Beasts) from the
// field, which are often face-down, and more of them on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (86132414, SZONE),
    (86132414, SZONE),
    (86132414, SZONE),
    (86132414, SZONE),
    (86132414, SZONE),
    (86132414, SZONE | OPPONENT),
    (86132414, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn exfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn cost_group(duel: &Duel, tp: u8) -> CardSet {
    let filter = if duel.is_player_affected_by_effect(tp, CARD_TRAP_EXTRA_COST).is_some() {
        exfilter
    } else {
        spfilter
    };
    duel.get_matching_group(filter, tp, LOCATION_ONFIELD, 0, None)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (duel.get_mzone_count(tp, Some(sg)) >= 1, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = cost_group(duel, tp);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && g.len() > 2
        && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = cost_group(duel, tp);
    let sg = aux::select_unselect_group_full(
        duel,
        g,
        3,
        3,
        &|duel, sg, _, _| rescon(duel, sg, tp),
        tp,
        HINTMSG_TOGRAVE,
        None,
        true,
    )?;
    let dg = sg.filter(duel, CardId::is_facedown, None);
    if !dg.is_empty() {
        duel.confirm_cards(1 - tp, dg)?;
    }
    if sg.len() == 3 {
        a.e.set_label_object_group(duel, Some(sg));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.send_to_grave(g, REASON_COST)?;
    }
    Ok(())
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_continuous_trap, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 1000
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_facedown(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_facedown, tp, 0, LOCATION_SZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_facedown, tp, 0, LOCATION_SZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_chain_limit(|duel, e, _, _, _| !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE), 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_facedown(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
