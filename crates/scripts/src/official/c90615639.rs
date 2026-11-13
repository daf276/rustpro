// Time Dimension Hole
use crate::prelude::*;

const ID: u32 = 90615639;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_DECK, 0) >= 2);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_any(tp)
            && duel.is_existing_matching_card(CardId::is_summonable_card, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_player_affected_by_effect(tp, CARD_EHERO_BLAZEMAN).is_none());
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) < 2 {
        return Ok(());
    }
    duel.shuffle_deck(tp)?;
    duel.confirm_decktop(tp, 1)?;
    let Some(tc) = duel.get_decktop_group(tp, 1).get_first() else {
        return Ok(());
    };
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && tc.is_summonable_card(duel)
        && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    {
        duel.disable_shuffle_check(true);
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    } else if duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])? == 1 {
        duel.move_sequence(tc, 1)?;
    }
    Ok(())
}
