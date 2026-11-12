// Nightwinged Cleric
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 79519259;
const ASSAULT_LICH: u32 = 86098176;

pub const LISTED_NAMES: &[u32] = &[CARD_ASSAULT_MODE];

// In the Extra Deck, Synchro Summoned with Genex Controller (a Level 3 Tuner) and Zanki (Level 5), with Assault Mode Activate and Assault Lich
// in the Deck and Sangan in hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68505803, MZONE),
    (68505803, MZONE),
    (30090452, MZONE),
    (30090452, MZONE),
    (CARD_ASSAULT_MODE, DECK),
    (CARD_ASSAULT_MODE, GRAVE),
    (ASSAULT_LICH, DECK),
    (ASSAULT_LICH, DECK),
    (26202165, HAND),
    (26202165, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::discard);
    e2.set_target(duel, thsptg);
    e2.set_operation(duel, thspop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_ASSAULT_MODE) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.is_existing_matching_card(setfilter, a.tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(setfilter),
        tp,
        LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first()
        && duel.sset(tp, sc)? > 0
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        sc.register_effect(duel, e1);
    }
    Ok(())
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mzone_chk: bool) -> bool {
    c.lists_code(duel, CARD_ASSAULT_MODE)
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (mzone_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mzone_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, mzone_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mzone_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(
        tp,
        |c, duel| thspfilter(c, duel, e, tp, mzone_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| mzone_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 4),
        )?;
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 5));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}
