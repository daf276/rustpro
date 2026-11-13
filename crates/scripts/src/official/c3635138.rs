// Emissaries from Darkness
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 3635138;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand to be summoned (and chained for a second copy), a Trap in the Graveyard to Set, and an attacker for the damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (4206964, GRAVE),
    (4206964, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_DAMAGE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |_, a| a.ep == a.tp && a.r & (REASON_BATTLE | REASON_EFFECT) > 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_status(duel, STATUS_SPSUMMON_TURN));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 2000);
    duel.set_possible_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.recover(tp, 2000, REASON_EFFECT)? > 0
    {
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let locations = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
        let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(&filter, tp, locations, 0, 1, None);
        let b2 = duel.is_existing_matching_card(CardId::is_attack_pos, tp, 0, LOCATION_MZONE, 1, None);
        if !((b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 2))?) {
            return Ok(());
        }
        let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
        if op == Some(1) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, &filter, tp, locations, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        } else if op == Some(2) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, CardId::is_attack_pos, tp, 0, LOCATION_MZONE, 1, 2, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && setfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(setfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, setfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SET, g, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) {
        duel.sset(tp, tc)?;
    }
    Ok(())
}
