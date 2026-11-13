// Underworld Circle
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 73443672;

// In the hand, with five Normal Monsters in each Graveyard to activate and Special Summon, and monsters on the field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (1184620, GRAVE),
    (1184620, GRAVE),
    (1184620, GRAVE),
    (1784619, GRAVE),
    (1784619, GRAVE),
    (1184620, GRAVE | OPPONENT),
    (1184620, GRAVE | OPPONENT),
    (1184620, GRAVE | OPPONENT),
    (1784619, GRAVE | OPPONENT),
    (1784619, GRAVE | OPPONENT),
    (1184620, MZONE),
    (1184620, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(CardId::is_monster, a.tp, LOCATION_GRAVE, 0, 5, None)
        && duel.is_existing_matching_card(CardId::is_monster, a.tp, 0, LOCATION_GRAVE, 5, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spnormalfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 {
        let rg = duel.get_matching_group(
            |c, duel| c.is_able_to_remove(duel) && c.is_monster(duel),
            tp,
            LOCATION_DECK,
            LOCATION_DECK,
            None,
        );
        if rg.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        let filter = aux::necro_valley_filter(|c, duel| spnormalfilter(c, duel, e, tp));
        if duel.remove(rg, POS_FACEDOWN, REASON_EFFECT)? > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_ALL, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn ignoresumconsp(duel: &mut Duel, e: EffectId, p: u8) -> Script<()> {
    if duel.get_location_count(p, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, p), p, LOCATION_GRAVE, 0, None);
    if g.is_empty() || !duel.select_yes_no(p, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, p, HINTMSG_SPSUMMON);
    if let Some(tc) = g.select(duel, p, 1, 1, None)?.get_first()
        && duel.special_summon_step(tc, 0, p, p, true, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        tc.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let turn_p = duel.get_turn_player();
    ignoresumconsp(duel, a.e, turn_p)?;
    ignoresumconsp(duel, a.e, 1 - turn_p)?;
    duel.special_summon_complete()?;
    Ok(())
}
