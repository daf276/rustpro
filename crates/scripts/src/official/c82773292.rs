// Indulged Darklord
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 82773292;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_DJEHUTY: u32 = 10426067;

// In the hand to Normal Summon, with Darklords of two Levels in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (DARKLORD_GULGOLET, DECK),
    (DARKLORD_GULGOLET, DECK),
    (DARKLORD_DJEHUTY, DECK),
    (DARKLORD_DJEHUTY, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_DARKLORD])
        && c.has_level(duel)
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
        && !c.is_level(duel, &[lv])
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_DARKLORD])
        && c.has_level(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff)
        && !c.is_code(duel, ID)
        && duel.is_existing_matching_card(
            |tc, duel| thfilter(tc, duel, c.get_level(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            Some(c.into()),
        )
}

fn opponent_zones(duel: &Duel, tp: u8) -> i32 {
    duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        return Ok(opponent_zones(duel, tp) > 0 && duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if opponent_zones(duel, tp) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        let g1 = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
        let Some(sc) = g1.get_first() else { return Ok(()) };
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let lv = sc.get_level(duel);
        let g2 = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, Some(g1))?;
        if g2.is_empty() {
            return Ok(());
        }
        duel.special_summon(g1, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)?;
        duel.send_to_hand(g2, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g2)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && !re.get_handler(duel).is_race(duel, RACE_FAIRY)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
