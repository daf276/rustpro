// Solfachord Solfegia
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, PZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 84693918;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_GRANSOLFACHORD, SET_SOLFACHORD];
// GranSolfachord Musecia on the field, and DoSolfachord Cutia in the hand to Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, PZONE),
    (37972500, MZONE),
    (55226153, HAND),
    (55226153, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

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
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        let not_faceup_solfachord = |c: CardId, duel: &Duel| !(c.is_faceup(duel) && c.is_set_card(duel, &[SET_SOLFACHORD]));
        !duel.is_existing_matching_card(not_faceup_solfachord, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.rp == 1 - tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_disablable(a.ev as u8)
        && !duel.has_flag_effect(tp, ID)
        && trig_loc == LOCATION_MZONE
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GRANSOLFACHORD])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.select_effect_yes_no_full(tp, c, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    duel.hint(HINT_CARD, 0, u64::from(ID));
    if duel.negate_effect(a.ev as u8) {
        duel.break_effect()?;
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, exc: Option<CardId>) -> bool {
    if !(c.is_set_card(duel, &[SET_SOLFACHORD]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_HAND | LOCATION_GRAVE) {
        duel.get_mzone_count(tp, exc.map(CardSet::from)) > 0
    } else if c.is_location(duel, LOCATION_EXTRA) {
        c.is_faceup(duel) && duel.get_location_count_from_ex(tp, tp, exc.map(CardSet::from), c) > 0
    } else {
        false
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cost_chk = Cost::self_tribute(duel, a, 0)?;
    let exc = cost_chk.then(|| e.get_handler(duel));
    let location = if cost_chk {
        LOCATION_HAND | LOCATION_GRAVE | LOCATION_EXTRA
    } else {
        LOCATION_HAND
    };
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, exc), tp, location, 0, 1, None));
    }
    let hand_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, None), tp, LOCATION_HAND, 0, 1, None);
    if cost_chk && (!hand_chk || duel.select_yes_no(tp, aux::stringid(ID, 3))?) {
        Cost::self_tribute(duel, a, 1)?;
        e.set_label(duel, i64::from(LOCATION_HAND | LOCATION_GRAVE | LOCATION_EXTRA));
    } else {
        e.set_label(duel, i64::from(LOCATION_HAND));
    }
    let label = e.get_label(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, label as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let location = e.get_label(duel) as u32;
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, None));
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
