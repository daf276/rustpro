// Fengli the Soldrapom
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 35311929;

// On the field with Dark Plant, to replace its destruction by Raigeki by sending a copy from the Deck, which Special Summons itself.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (13193642, MZONE),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// The replacement is a continuous effect that forms no chain link: its prompt is checked.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, desreptg);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_DECK)
        && a.r & REASON_EFFECT > 0
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_race(duel, RACE_PLANT)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            Some(c.into()),
        )
        && duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
        let g = duel.select_matching_card(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            duel.break_effect()?;
            duel.hint_selection(g);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_value(duel, tc.get_attack(duel) / 2);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
            e2.set_value(duel, tc.get_defense(duel) / 2);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn desrepfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PLANT) && c.is_able_to_grave(duel)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && (c.is_reason(duel, REASON_BATTLE) || c.is_reason(duel, REASON_EFFECT))
            && duel.is_existing_matching_card(desrepfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, desrepfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_EFFECT | REASON_REPLACE)?;
    Ok(true)
}
