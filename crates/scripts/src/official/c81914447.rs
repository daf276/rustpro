// Noh-P.U.N.K. Ogre Dance
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 81914447;

pub const LISTED_SERIES: &[u16] = &[SET_PUNK];

// In the hand with Ze Amin on the field to release and in the Deck to add, and on the field against the opponent's Ze Amin
// activating its search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (19535693, MZONE),
    (19535693, DECK),
    (19535693, DECK),
    (19535693, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atkcon);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| a.rp == 1 - a.tp && re.is_monster_effect(duel) && re.get_handler(duel).get_base_attack(duel) > 0)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(rc) = a.re.map(|re| re.get_handler(duel)) else {
        return Ok(());
    };
    let c = e.get_handler(duel);
    let atk = rc.get_base_attack(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && atk > 0 && rc.is_controler(duel, 1 - tp) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn punk(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PUNK])
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, punk, (1, 1), false, Some(&aux::release_check_mmz), None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_release_group_cost(tp, punk, (1, 1), false, Some(&aux::release_check_mmz), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, a.tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_HAND as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_PUNK]) && !c.is_level(duel, &[8]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
