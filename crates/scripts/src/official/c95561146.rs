// Apophis the Serpent
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 95561146;

pub const LISTED_NAMES: &[u32] = &[ID, 28649820];
// Set and in the hand, with Embodiment of Apophis and Apophis the Swamp Deity in the Deck to search and Set, and a Raigeki
// in the opponent's hand to destroy it as a monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (28649820, DECK),
    (28649820, DECK),
    (85888377, DECK),
    (85888377, DECK),
    (12580477, HAND | OPPONENT),
    (12580477, DECK | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(
        SET_APOPHIS,
        TYPE_MONSTER | TYPE_NORMAL,
        1600,
        1800,
        4,
        RACE_REPTILE,
        ATTRIBUTE_EARTH,
    );
    duel.is_player_can_special_summon_monster(tp, ID, stats)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_APOPHIS]) && c.is_trap(duel) && c.is_ssetable(duel) && !c.is_code(duel, ID)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon(duel, tp) {
        return Ok(());
    }
    c.add_monster_attribute(duel, TYPE_NORMAL | TYPE_TRAP);
    duel.special_summon_step(c, 1, tp, tp, true, false, POS_FACEUP)?;
    c.add_monster_attribute_complete(duel);
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    if duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let Some(sc) = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?.first() else {
            return Ok(());
        };
        duel.break_effect()?;
        if duel.sset(tp, sc)? > 0 {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 3));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
            e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, 28649820) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
