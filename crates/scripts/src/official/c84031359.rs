// Darklord Gulgolet
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 84031359;
const TOKEN_DARKLORD: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[ID, TOKEN_DARKLORD];

// On the field and in the Graveyard, with Monster Reborn and Dark Hole in the hand and Darklord Desire to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (83764718, HAND),
    (53129443, HAND),
    (55690251, DECK),
    (55690251, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_DARKLORD),
];

const TOKEN_STATS: MonsterStats = MonsterStats::new(SET_DARKLORD, TYPES_TOKEN, 0, 0, 6, RACE_FAIRY, ATTRIBUTE_DARK);

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn can_summon_tokens(duel: &mut Duel, tp: u8) -> bool {
    duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) >= 2
        && duel.is_player_can_special_summon_monster(tp, TOKEN_DARKLORD, TOKEN_STATS)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_tokens(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_FAIRY));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_FAIRY));
    if !can_summon_tokens(duel, tp) {
        return Ok(());
    }
    for _ in 0..2 {
        if let Some(token) = duel.create_token(tp, TOKEN_DARKLORD) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_DARKLORD]) || (c.is_set_card(duel, &[SET_FORBIDDEN]) && c.is_quick_play_spell(duel)))
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
