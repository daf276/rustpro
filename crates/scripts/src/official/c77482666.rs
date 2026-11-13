// Swiftwind Panther Warrior
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 77482666;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_TIME_WIZARD, ID];

// On the field with Normal Monsters to Tribute, and Alligator's Dragon Knight and Dark Time Wizard in the Deck to summon and send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1784619, MZONE),
    (1784619, HAND),
    (4881365, DECK),
    (4881365, DECK),
    (40235813, DECK),
    (40235813, DECK),
    (65118318, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(4881365)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| duel.get_flag_effect(0, ID) == 0);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_RELEASE);
        ge1.set_operation(duel, |duel, a| {
            if duel.get_flag_effect(0, ID) == 0
                && a.eg.is_some_and(|eg| {
                    eg.is_exists(
                        duel,
                        |c, duel| {
                            c.is_previous_location(duel, LOCATION_MZONE)
                                || (!c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_monster_card(duel))
                        },
                        1,
                        None,
                    )
                })
            {
                duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
            }
            Ok(())
        });
        duel.register_effect(ge1, 0);
    });
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_DECK,
            0,
            1,
            Some(c.into()),
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.lists_code(duel, CARD_DARK_TIME_WIZARD) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_DARK_TIME_WIZARD) && c.is_able_to_grave(duel)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let b1 = duel.check_release_group_cost(tp, |tc, duel| spcostfilter(tc, duel, e, tp), (1, 1), true, None, Some(c.into()));
    let b2 = duel.check_release_group_cost(tp, aux::TRUE, (1, 1), true, None, Some(c.into()))
        && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let g = if b2 {
        duel.select_release_group_cost(tp, aux::TRUE, (1, 1), true, None, Some(c.into()))?
    } else {
        duel.select_release_group_cost(tp, |tc, duel| spcostfilter(tc, duel, e, tp), (1, 1), true, None, Some(c.into()))?
    };
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_TOGRAVE);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
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
    } else if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
