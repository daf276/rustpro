// Elfnotes: Rhapsodia of Madness
use crate::prelude::*;

const ID: u32 = 24092792;

pub const LISTED_SERIES: &[u16] = &[SET_ELFNOTE];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_card(a.e.get_handler_player(duel), LOCATION_MZONE, 2).is_some()
    });
    e1.set_value(duel, Value::Card(|duel, _, c| c.is_sequence(duel, 2).into()));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let attr = c.get_original_attribute(duel);
    c.is_monster(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_target(|sc, duel| spfilter(sc, duel, e, tp, attr), tp, LOCATION_GRAVE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, attr: u32) -> bool {
    c.is_set_card(duel, &[SET_ELFNOTE])
        && !c.is_original_attribute(duel, attr)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(false);
    };
    e.set_label(duel, sc.get_original_attribute(duel).into());
    duel.send_to_grave(sc, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let attr = e.get_label(duel) as u32;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp, attr));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, attr), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_synchro_monster(duel)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
        && duel.is_existing_matching_card(|c, duel| c.is_negatable(duel), tp, 0, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        let sc = duel.select_matching_card(tp, |c, duel| c.is_negatable(duel), tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if let Some(sc) = sc.get_first() {
            duel.hint_selection(sc);
            duel.break_effect()?;
            sc.negate_effects(duel, e.get_handler(duel), RESETS_STANDARD_PHASE_END, true);
        }
    }
    Ok(())
}
