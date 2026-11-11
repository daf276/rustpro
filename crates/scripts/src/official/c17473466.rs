// Nervedo the Shadebeast Power Patron
use super::c10266279::mdescost as descost;
use crate::prelude::*;
use crate::testing::Check;

const ID: u32 = 17473466;
const NERVA: u32 = 53589300;

pub const LISTED_NAMES: &[u32] = &[ID, NERVA];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, discon);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, descost));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_DECK);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let ch = i32::from(duel.get_current_chain()) - 1;
    let Ok(ch) = u8::try_from(ch) else { return false };
    ch > 0
        && a.ep == 1 - tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && !duel.has_flag_effect(tp, ID)
        && duel.is_chain_disablable(a.ev as u8)
        && chain::is_triggering_controler(duel, ch, tp)
        && chain::is_triggering_type(duel, ch, TYPE_MONSTER)
        && chain::is_triggering_setcode(duel, ch, &[SET_POWER_PATRON, SET_ARTMAGE])
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.select_effect_yes_no_full(tp, c, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    duel.hint(HINT_CARD, 0, ID.into());
    if duel.negate_effect(a.ev as u8) {
        duel.break_effect()?;
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn nervaspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, hc: Option<CardId>) -> bool {
    c.is_code(duel, NERVA)
        && duel.get_location_count_from_ex(tp, tp, hc.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let filter = |sc, duel: &Duel| nervaspfilter(sc, duel, e, tp, Some(c));
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.destroy(c, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |sc, duel: &Duel| nervaspfilter(sc, duel, e, tp, None);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    sc.set_material(duel, None);
    if duel.special_summon(sc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !c.is_set_card(duel, &[SET_POWER_PATRON, SET_ARTMAGE])
        || c.is_code(duel, ID)
        || !c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    {
        return false;
    }
    if c.is_location(duel, LOCATION_DECK) {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    } else {
        c.is_faceup(duel) && duel.get_location_count_from_ex(tp, tp, None, c) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
