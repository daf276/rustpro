// Spell Power Attainment
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 39123673;

pub const LISTED_SERIES: &[u16] = &[SET_ENDYMION];
pub const LISTED_NAMES: &[u32] = &[ID];
pub const COUNTER_LIST: &[u16] = &[COUNTER_SPELL];

// Cards that mention Spell Counters (Apprentice Magician, Explosive Magician) in the Deck, hand and Graveyard, with Skilled White Magician
// to hold the counters Apprentice Magician places, a Level 7 Endymion (Master Magician) and the opponent's monsters to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (9156135, HAND),
    (9156135, HAND),
    (9156135, DECK),
    (9156135, DECK),
    (33413279, GRAVE),
    (46363422, MZONE),
    (40732515, MZONE),
    (3985011, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_counter(duel, COUNTER_SPELL) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn endymionfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 7) && c.is_set_card(duel, &[SET_ENDYMION]) && c.is_monster_card(duel) && c.is_faceup(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(sc) = duel
        .select_matching_card(
            tp,
            aux::necro_valley_filter(thfilter),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?
        .get_first()
    else {
        return Ok(());
    };
    if sc.is_location(duel, LOCATION_GRAVE) {
        duel.hint_selection(sc);
    }
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? == 0 || !sc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    if sc.is_previous_location(duel, LOCATION_DECK) {
        duel.confirm_cards(1 - tp, sc)?;
    }
    duel.shuffle_hand(tp)?;
    if !duel.is_existing_matching_card(endymionfilter, tp, LOCATION_ONFIELD, 0, 1, None)
        || !duel.is_can_remove_counter(tp, 1, 0, COUNTER_SPELL, 1, REASON_EFFECT)
    {
        return Ok(());
    }
    let ng = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    if ng.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    let mut max_ct = 1;
    for i in 2..=ng.len() {
        if !duel.is_can_remove_counter(tp, 1, 0, COUNTER_SPELL, i as u16, REASON_EFFECT) {
            break;
        }
        max_ct = i;
    }
    let ct = if max_ct == 1 {
        1
    } else {
        duel.announce_number_range(tp, 1, max_ct as u32)? as usize
    };
    duel.break_effect()?;
    if !duel.remove_counter(tp, 1, 0, COUNTER_SPELL, ct as u16, REASON_EFFECT)? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let mut sng = ng.select(duel, tp, ct as u8, ct as u8, None)?;
    if sng.len() == ct {
        duel.hint_selection(sng);
        sng.keep_matching(duel, |c, duel| c.is_can_be_disabled_by_effect(duel, e, true), None);
        let c = e.get_handler(duel);
        for tc in sng {
            tc.negate_effects(duel, c, 0, false);
        }
        duel.adjust_instantly(None);
        duel.destroy(sng, REASON_EFFECT)?;
    }
    Ok(())
}
