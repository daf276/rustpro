// Mitsurugi Prayers
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 45171524;

pub const LISTED_SERIES: &[u16] = &[SET_MITSURUGI];
// Aramasa (Reptile) on the field to Tribute and in the Deck, Graveyard and hand to add and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (40543231, MZONE),
    (40543231, DECK),
    (40543231, DECK),
    (40543231, GRAVE),
    (40543231, HAND),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(40543231)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DAMAGE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn reptile(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    if duel.check_release_group_cost(tp, reptile, (1, 1), true, None, None) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        let g = duel.select_release_group_cost(tp, reptile, (1, 1), true, None, None)?;
        duel.release(g, REASON_COST)?;
        e.set_label(duel, 1);
    } else {
        e.set_label(duel, 0);
    }
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 1, tp, 800);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MITSURUGI]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MITSURUGI]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let both = b1 && e.get_label(duel) == 1;
    let op = duel.select_effect(
        tp,
        &[
            (b1, aux::stringid(ID, 2)),
            (true, aux::stringid(ID, 3)),
            (both, aux::stringid(ID, 4)),
        ],
    )?;
    let mut breakeffect = false;
    if op == Some(1) || op == Some(3) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            breakeffect = true;
        }
    }
    if op == Some(2) || op == Some(3) {
        if breakeffect {
            duel.break_effect()?;
        }
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let locations = LOCATION_HAND | LOCATION_GRAVE;
        if duel.damage(tp, 800, REASON_EFFECT)? > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(&filter, tp, locations, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 5))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sc = duel.select_matching_card(tp, &filter, tp, locations, 0, 1, 1, None)?.get_first();
            duel.break_effect()?;
            if let Some(sc) = sc
                && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
            {
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_description(duel, 3207);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                sc.register_effect(duel, e1);
            }
            duel.special_summon_complete()?;
        }
    }
    Ok(())
}
