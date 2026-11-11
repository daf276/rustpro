// Hamon, Lord of Striking Thunder
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 32491822;
const CARD_SPELL_EXTRA_COST: u32 = 54828837;

// Copies in the hand, Special Summoned by sending three Continuous Spells (Sacred Beasts Thunderclap) from the field, and
// on the field with monsters of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (1259915, SZONE),
    (1259915, SZONE),
    (1259915, SZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
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
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_condition(duel, damcon);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, stock::damage);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    e4.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_continuous_spell(duel) && c.is_able_to_grave_as_cost(duel)
}

fn exfilter(c: CardId, duel: &Duel) -> bool {
    spfilter(c, duel) || (c.is_facedown(duel) && c.is_spell(duel) && c.is_able_to_grave_as_cost(duel))
}

fn cost_group(duel: &Duel, tp: u8) -> CardSet {
    let filter = if duel.is_player_affected_by_effect(tp, CARD_SPELL_EXTRA_COST).is_some() {
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

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_battle_target(duel).is_some_and(|bc| {
        c.is_relate_to_battle(duel) && bc.is_location(duel, LOCATION_GRAVE) && bc.is_reason(duel, REASON_BATTLE) && bc.is_monster(duel)
    })
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    Ok(false)
}
