// Noh-P.U.N.K. Rising Scale
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 75046994;

pub const LISTED_SERIES: &[u16] = &[SET_PUNK];

// In the hand with P.U.N.K. monsters to banish, Ze Amin in the Deck to add or summon, and the opponent's Foxy Tune and Ze Amin
// for a monster effect activated at 2500 ATK or more.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (19535693, HAND),
    (19535693, GRAVE),
    (19535693, DECK),
    (19535693, DECK),
    (55920742, MZONE | OPPONENT),
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
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::pay_lp::<600>);
    e2.set_target(duel, thsptg);
    e2.set_operation(duel, thspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, poscon);
    e3.set_target(duel, postg);
    e3.set_operation(duel, posop);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PUNK])
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = Some(a.e.get_handler(duel).into());
    let location = LOCATION_HAND | LOCATION_GRAVE | LOCATION_MZONE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(spcostfilter, tp, location, 0, 1, c));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let tc = duel.select_matching_card(tp, spcostfilter, tp, location, 0, 1, 1, c)?;
    duel.remove(tc, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sp_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_PUNK])
        && c.is_monster(duel)
        && !c.is_level(duel, &[8])
        && (c.is_able_to_hand(duel) || (sp_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let sp_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, sp_chk);
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    let location = (LOCATION_DECK | LOCATION_GRAVE) as i32;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, location);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(|c, duel| thspfilter(c, duel, e, tp, sp_chk));
    let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        tc,
        tp,
        |tc, duel| sp_chk && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 3),
    )
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let atk: i64 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_ATTACK);
    a.re.is_some_and(|re| {
        loc == LOCATION_MZONE && re.is_monster_effect(duel) && re.get_handler(duel).is_controler(duel, 1 - a.tp) && atk >= 2500
    })
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else {
        return Ok(false);
    };
    let rc = re.get_handler(duel);
    if chk == 0 {
        return Ok(rc.is_faceup(duel) && rc.is_can_turn_set(duel) && rc.is_relate_to_effect(duel, re));
    }
    duel.set_operation_info(0, CATEGORY_POSITION, Some(rc.into()), 1, a.tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_relate_to_effect(duel, re) {
            duel.change_position(rc, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}
