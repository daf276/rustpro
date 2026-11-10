// Eater of Millions
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 63845230;

// In the hand with Mokey Mokey in the hand and on the field to banish face-down, and Elfnote Lucina of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (27288416, HAND),
    (27288416, HAND),
    (27288416, HAND),
    (27288416, MZONE),
    (27288416, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::Card(val));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_value(duel, 1);
    c.register_effect(duel, e5);
    let e6 = e5.clone_effect(duel);
    e6.set_code(duel, EFFECT_UNRELEASABLE_NONSUM);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE);
    e7.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e7.set_value(
        duel,
        Value::Material(|_, _, _, sumtype, _| {
            aux::cannotmatfilter(&[SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ].map(u64::from), sumtype)
        }),
    );
    c.register_effect(duel, e7);
    let e8 = duel.create_effect(c);
    e8.set_description(duel, aux::stringid(ID, 0));
    e8.set_category(duel, CATEGORY_REMOVE);
    e8.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e8.set_code(duel, EVENT_BATTLE_START);
    e8.set_count_limit(duel, 1, 0);
    e8.set_target(duel, rmtg);
    e8.set_operation(duel, rmop);
    c.register_effect(duel, e8);
}

fn removable_as_cost(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let rg = duel.get_matching_group(
        removable_as_cost,
        tp,
        LOCATION_HAND | LOCATION_ONFIELD | LOCATION_EXTRA,
        0,
        Some(c.into()),
    );
    duel.get_mzone_count(tp, Some(rg)) > 0 && rg.len() > 4 && aux::select_unselect_group_check(duel, rg, 5, rg.len(), &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let location = LOCATION_HAND | LOCATION_ONFIELD | LOCATION_EXTRA;
    let rg = duel.get_matching_group(removable_as_cost, tp, location, 0, Some(c.into()));
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = if ft <= 0 {
        let mut g1 = duel.select_matching_card_full(tp, removable_as_cost, tp, LOCATION_MMZONE, 0, 1, 1, true, Some(c.into()))?;
        let Some(first) = g1.get_first() else { return Ok(false) };
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let exception = CardSet::from_iter([c, first]);
        let g2 = duel.select_matching_card_full(tp, removable_as_cost, tp, location, 0, 4, rg.len() as u8, true, Some(exception))?;
        g1.merge(g2);
        if g2.is_empty() {
            return Ok(false);
        }
        g1
    } else {
        duel.select_matching_card_full(tp, removable_as_cost, tp, location, 0, 5, rg.len() as u8, true, Some(c.into()))?
    };
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEDOWN, REASON_COST)?;
    }
    Ok(())
}

fn val(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_facedown, 0, LOCATION_REMOVED, LOCATION_REMOVED, None) as i32 * 100
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(
            tc.is_some_and(|tc| tc.is_controler(duel, 1 - tp) && tc.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT))
        );
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel)
        && tc.is_relate_to_battle(duel)
    {
        duel.remove(tc, POS_FACEDOWN, REASON_EFFECT)?;
    }
    Ok(())
}
