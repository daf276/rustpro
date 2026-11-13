// Ashtra Insect Poison
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 64455720;
const CHOICE: u8 = 0;

// In the hand to be Special Summoned as a Tuner for Ashtraja Susanoo (Level 9) with Akasuna (Level 8) on the field, or to
// revive Yoa and Akasuna from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (38331244, MZONE),
    (1710647, GRAVE),
    (38331244, GRAVE),
    (17020474, EXTRA),
    (43419178, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(ID),
    Check::SpecialSummonedCode(1710647),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
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
        RACE_INSECT,
        ATTRIBUTE_DARK,
    )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ASHTRA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let b1 = mmz_chk && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && duel.is_player_can_special_summon_monster(tp, ID, stats());
    let b2 = mmz_chk && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    match choice {
        Some(1) => {
            e.set_property(duel, 0);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(e.get_handler(duel).into()), 1, tp, 0);
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
        }
        Some(2) => {
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
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
            if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) {
                duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        _ => {}
    }
    Ok(())
}
