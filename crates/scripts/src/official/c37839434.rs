// Ashtra Divine Domain
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 37839434;
const CHOICE: u8 = 0;
const NEGATE_CHAIN_LINK: u8 = 1;

// Set and in the hand, to be Special Summoned as a Tuner for Ashtraja Susanoo (Level 9) and Ashtraya Mikumari-no-Mikoto (Level 8)
// with Akasuna (Level 8) and Yoa (Level 7) on the field, or to negate the opponent's Monster Reborn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (38331244, MZONE),
    (1710647, MZONE),
    (17020474, EXTRA),
    (43419178, EXTRA),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(ID),
    Check::Message(MSG_CHAIN_DISABLED),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn stats() -> MonsterStats {
    MonsterStats::new(
        SET_ASHTRA,
        TYPE_MONSTER | TYPE_NORMAL | TYPE_TUNER,
        0,
        0,
        1,
        RACE_AQUA,
        ATTRIBUTE_WATER,
    )
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && duel.is_player_can_special_summon_monster(tp, ID, stats());
    let event = duel.check_event_full(EVENT_CHAINING);
    let b2 = event.is_some_and(|event| {
        event.event_player == 1 - tp
            && event.reason_effect.is_some_and(|re| re.is_spell_trap_effect(duel))
            && duel.is_chain_disablable(event.event_value as u8)
            && !duel.is_existing_matching_card(CardId::is_facedown, tp, 0, LOCATION_ONFIELD, 1, None)
    });
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    match choice {
        Some(1) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(e.get_handler(duel).into()), 1, tp, 0);
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
        }
        Some(2) => {
            if let Some(event) = event {
                e.set_chain_data(duel, NEGATE_CHAIN_LINK, i64::from(event.event_value));
                e.set_category(duel, CATEGORY_DISABLE);
                let rc = event.reason_effect.map(|re| re.get_handler(duel).into());
                duel.set_operation_info(0, CATEGORY_DISABLE, rc, 1, tp, 0);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            let c = e.get_handler(duel);
            if c.is_relate_to_effect(duel, e) && duel.is_player_can_special_summon_monster(tp, ID, stats()) {
                c.add_monster_attribute(duel, TYPE_NORMAL | TYPE_TUNER);
                duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
                c.add_monster_attribute_complete(duel);
            }
            if duel.special_summon_complete()? == 0 {
                return Ok(());
            }
            let g = duel.get_matching_group(|c, duel| c.is_synchro_summonable(duel, None, None), tp, LOCATION_EXTRA, 0, None);
            if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let sg = g.select(duel, tp, 1, 1, None)?;
                if let Some(sc) = sg.get_first() {
                    duel.synchro_summon(tp, sc, None, None)?;
                }
            }
        }
        Some(2) => {
            if let Some(link) = e.get_chain_data(duel, NEGATE_CHAIN_LINK) {
                duel.negate_effect(link as u8);
            }
        }
        _ => {}
    }
    Ok(())
}
