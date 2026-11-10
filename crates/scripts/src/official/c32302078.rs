// Dhampir Vampire Sheridan
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 32302078;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// In the Extra Deck, Xyz Summoned with four Summoned Skulls (Level 6), with Raigeki in the hand to send the opponent's Sangans
// to the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (70781052, MZONE),
    (70781052, MZONE),
    (70781052, MZONE),
    (70781052, MZONE),
    (12580477, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_XYZ_LEVEL);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_range(duel, LOCATION_EXTRA);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| {
        c.has_level(duel) && c.get_owner(duel) != e.get_handler_player(duel)
    });
    e1.set_value(
        duel,
        Value::XyzLevel(|duel, _, c, rc| if rc.is_code(duel, ID) { 6 } else { c.get_level(duel) as i32 }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CUSTOM + ID);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    e3.set_label_object_group(duel, Some(CardSet::default()));
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3a.set_code(duel, EVENT_TO_GRAVE);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_label_object(duel, Some(e3));
    e3a.set_operation(duel, regop);
    c.register_effect(duel, e3a);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.is_controler(duel, 1 - tp)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let group = e.get_label_object_group(duel).unwrap_or_default();
    let g = group.filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if chk == 0 {
        return Ok(!g.is_empty() && duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = e.get_label_object_group(duel).unwrap_or_default();
    if g.is_empty() || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let sg = g.filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp), 1, 1, None)?;
    if !sg.is_empty() {
        duel.hint_selection(sg);
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e3) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e3.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e3.set_label_object_group(duel, Some(g));
    if !g.is_empty() && !duel.has_flag_effect(tp, ID) {
        duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
        duel.raise_event(g, EVENT_CUSTOM + ID, a.re, a.r, tp, a.ep, a.ev)?;
    }
    Ok(())
}
