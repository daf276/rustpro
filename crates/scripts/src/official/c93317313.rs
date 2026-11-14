// D/D Defense Soldier
use crate::prelude::*;
use crate::testing::{Check, GRAVE, PZONE};

const ID: u32 = 93317313;

pub const LISTED_SERIES: &[u16] = &[SET_DD, SET_DDD];
// D/D Orthros in the Pendulum Zone and the Graveyard, as summon target, banish cost and search target.
pub const SUPPORT: &[(u32, u16)] = &[(72181263, PZONE), (72181263, GRAVE), (72181263, GRAVE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn limcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_PZONE, 0, 1, None);
    let b2 = duel.get_flag_effect(tp, ID) == 0 && duel.is_existing_matching_card(limcostfilter, tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_PZONE, 0, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, 0);
        e.set_property(duel, 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, limcostfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if let Some(tc) = duel.get_first_target()
                && tc.is_relate_to_effect(duel, e)
            {
                duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        2 => {
            if duel.get_flag_effect(tp, ID) > 0 {
                return Ok(());
            }
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, aux::stringid(ID, 4));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
            e1.set_range(duel, LOCATION_MZONE);
            e1.set_target_range(duel, 0, 1);
            e1.set_condition(duel, actcon);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        }
        _ => {}
    }
    Ok(())
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker()
        .is_some_and(|tc| tc.is_set_card(duel, &[SET_DDD]) && tc.is_controler(duel, a.e.get_handler_player(duel)))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_EXTRA | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_EXTRA | LOCATION_GRAVE)
}
