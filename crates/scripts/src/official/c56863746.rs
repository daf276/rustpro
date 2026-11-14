// Drytron Meteonis DA Draconids
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 56863746;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
pub const LISTED_NAMES: &[u32] = &[22398665];
// Ritual Summoned by Meteonis Drytron from Drytron monsters, with more in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (22398665, HAND),
    (22398665, HAND),
    (97148796, HAND),
    (96026108, MZONE),
    (60037599, MZONE),
    (22420202, MZONE),
    (97148796, GRAVE),
    (96026108, GRAVE),
    (69815951, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| c != e.get_handler(duel) && c.is_set_card(duel, &[SET_DRYTRON]));
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 2, 0);
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp
            && a.re.is_some_and(|re| re.is_monster_effect(duel))
            && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
            && duel.is_chain_negatable(a.ev as u8)
    });
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRYTRON]) && c.is_monster(duel) && c.get_attack(duel) > 0 && c.is_able_to_remove_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let atk = a.re.map_or(0, |re| re.get_handler(duel).get_base_attack(duel));
    let g = duel.get_matching_group(negcostfilter, tp, LOCATION_GRAVE, 0, None);
    let attack = |c: CardId, duel: &Duel| c.get_attack(duel) as u32;
    if chk == 0 {
        return Ok(!g.is_empty() && g.check_with_sum_greater(duel, attack, atk as u32));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = match atk > 0 {
        true => g.select_with_sum_greater(duel, tp, attack, atk as u32)?,
        false => g.select(duel, tp, 1, 1, None)?,
    };
    duel.remove(rg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, Some(c), None, REASON_RITUAL);
    pg.is_empty()
        && c.is_ritual_monster(duel)
        && c.is_attack(duel, &[4000])
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_RITUAL, tp, true, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp) && c.is_previous_location(duel, LOCATION_MZONE) && c.is_ritual_summoned(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND | LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
        && duel.special_summon(tc, SUMMON_TYPE_RITUAL, tp, tp, true, false, POS_FACEUP)? > 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}
