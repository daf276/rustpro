// Girsu, the Orcust Mekk-Knight
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 69811710;

pub const LISTED_NAMES: &[u32] = &[TOKEN_WORLD_LEGACY];

// In the hand and on the field, with copies in the Deck to send to the Graveyard (an Orcust card).
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (ID, DECK), (ID, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_WORLD_LEGACY),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
    });
    e3.set_target(duel, tktg);
    e3.set_operation(duel, tkop);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ORCUST, SET_WORLD_LEGACY]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_grave(tc, REASON_EFFECT)? <= 0 || !tc.is_location(duel, LOCATION_GRAVE) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.get_column_group_count(duel) >= 2 {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn can_summon_tokens(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(SET_WORLD_LEGACY, TYPES_TOKEN, 0, 0, 1, RACE_MACHINE, ATTRIBUTE_DARK);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.is_player_can_special_summon_monster_full(tp, TOKEN_WORLD_LEGACY, stats, POS_FACEUP_DEFENSE, tp, 0)
        && duel.is_player_can_special_summon_monster_full(tp, TOKEN_WORLD_LEGACY, stats, POS_FACEUP_DEFENSE, 1 - tp, 0)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_tokens(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if can_summon_tokens(duel, tp) {
        let (Some(tk1), Some(tk2)) = (duel.create_token(tp, TOKEN_WORLD_LEGACY), duel.create_token(tp, TOKEN_WORLD_LEGACY)) else {
            return Ok(());
        };
        duel.special_summon_step(tk1, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        duel.special_summon_step(tk2, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)?;
        duel.special_summon_complete()?;
    }
    Ok(())
}
