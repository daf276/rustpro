// Ultimate Sky
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 38817295;
const CHOICE: u8 = 0;
const EVENT_VALUE: u8 = 1;

// Opponent's monsters to out-number the field and to target, and Ash Blossom (a hand effect) and Sangan (a Graveyard effect, with my
// Raigeki to send it) for the second option.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (3985011, MZONE | OPPONENT),
    (3985011, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (14558127, HAND | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > duel.get_field_group_count(a.tp, LOCATION_MZONE, 0)
    });
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(e.get_chain_data(duel, CHOICE) == Some(1) && chkc.is_negatable(duel) && chkc.is_on_field(duel) && chkc != c);
    }
    let negate_group = duel.get_target_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    let opp_faceup_monster_count = duel.get_matching_group_count(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let option_1 = !duel.has_flag_effect(tp, ID) && !negate_group.is_empty() && opp_faceup_monster_count > 0 && duel.check_lp_cost(tp, 800);
    let event = duel.check_event_full(EVENT_CHAINING);
    let option_2 = !duel.has_flag_effect(tp, ID + 1)
        && event.is_some_and(|event| {
            event.event_player == 1 - tp
                && event.reason_effect.is_some_and(|re| re.is_monster_effect(duel))
                && chain::is_triggering_location(duel, event.event_value as u8, LOCATION_HAND | LOCATION_GRAVE)
                && duel.is_chain_disablable(event.event_value as u8)
        });
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 1)), (option_2, aux::stringid(ID, 2))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        let max_target_count = negate_group.len().min(opp_faceup_monster_count).min(duel.get_lp(tp) as usize / 800);
        let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| (duel.check_lp_cost(tp, 800 * sg.len() as u32), false);
        let tg = aux::select_unselect_group_full(
            duel,
            negate_group,
            1,
            max_target_count,
            &rescon,
            tp,
            HINTMSG_NEGATE,
            Some(&rescon),
            false,
        )?;
        duel.set_target_card(tg);
        duel.pay_lp_cost(tp, 800 * tg.len() as u32)?;
        duel.set_operation_info(0, CATEGORY_DISABLE, Some(tg), tg.len() as u8, tp, 0);
    } else if choice == Some(2)
        && let Some(event) = event
    {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_property(duel, 0);
        e.set_chain_data(duel, EVENT_VALUE, i64::from(event.event_value));
        duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            let Some(tg) = duel.get_target_cards(e).filter(|tg| !tg.is_empty()) else {
                return Ok(());
            };
            let c = e.get_handler(duel);
            for tc in tg {
                tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, true);
            }
        }
        Some(2) => {
            duel.negate_effect(e.get_chain_data(duel, EVENT_VALUE).unwrap_or(0) as u8);
        }
        _ => {}
    }
    Ok(())
}
