// Mudan the Rikka Fairy
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 71002019;

pub const LISTED_SERIES: &[u16] = &[SET_RIKKA];
// In hand with Plants (Snakeyashi) on the field to Tribute, and Rikka Spells/Traps (Glamour, Sheet) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (29802344, MZONE),
    (29802344, MZONE),
    (69164989, DECK),
    (69164989, DECK),
    (68941332, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, thcon);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_rikka_releasable(duel, tp) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, a.tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.get_location_count(a.tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RIKKA]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel) && re.get_handler(duel).is_race(duel, RACE_PLANT))
}
