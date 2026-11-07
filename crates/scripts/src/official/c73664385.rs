// Verre Magic - Lacrima of Light
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 73664385;
// The field of its chain data table.
const CHOICE: u8 = 0;

// In the hand with a "Magistus" monster on the field. No Normal Monster or ported card is a "Magistus" or "Witchcrafter"
// monster that can be Special Summoned from the hand or Deck, so the second effect never resolves.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (66532962, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_spell(duel) || c.is_race(duel, RACE_SPELLCASTER)) && c.is_able_to_grave(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAGISTUS, SET_WITCHCRAFTER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    let b1 = !duel.has_flag_effect(tp, ID)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_MAGISTUS, SET_WITCHCRAFTER])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
        && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        && duel
            .check_event_full(EVENT_CHAINING)
            .is_some_and(|event| event.event_player == 1 - tp);
    Ok(chk == 0 && (b1 || b2))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cost_skip = e.get_label(duel) != -100;
    let b1 = (cost_skip
        || (!duel.has_flag_effect(tp, ID)
            && duel.is_existing_matching_card(
                aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_MAGISTUS, SET_WITCHCRAFTER])),
                tp,
                LOCATION_MZONE,
                0,
                1,
                None,
            )))
        && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = (cost_skip
        || (!duel.has_flag_effect(tp, ID + 1)
            && duel
                .check_event_full(EVENT_CHAINING)
                .is_some_and(|event| event.event_player == 1 - tp)))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    if choice == Some(1) {
        e.set_category(duel, CATEGORY_TOGRAVE);
        if !cost_skip {
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        if !cost_skip {
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_grave(g, REASON_EFFECT)?;
            }
        }
        Some(2) => {
            if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(
                tp,
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK,
                0,
                1,
                1,
                None,
            )?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        _ => {}
    }
    Ok(())
}
