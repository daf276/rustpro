// Rustin Mammoth
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 3434362;

// In hand with Cyber Dragon Sieger (Link-2) and Sky Striker Ace - Azalea Temperance (Link-3) in the Extra Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (46724542, EXTRA), (56741506, EXTRA)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND);
    e2a.set_type(duel, EFFECT_TYPE_IGNITION);
    e2a.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_condition(duel, |duel, a| !rthquickcon(duel, a));
    e2a.set_target(duel, rthtg);
    e2a.set_operation(duel, rthop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2b.set_code(duel, EVENT_FREE_CHAIN);
    e2b.set_condition(duel, rthquickcon);
    e2b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2b);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_link_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.get_sum(duel, |c, duel| i64::from(c.get_link(duel))) == 5, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_EXTRA, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty() && aux::select_unselect_group_check(duel, g, 1, 5, &rescon));
    }
    let rg = aux::select_unselect_group_full(duel, g, 1, 5, &rescon, tp, HINTMSG_REMOVE, Some(&rescon), false)?;
    duel.remove(rg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.get_location_count_full(tp, LOCATION_MZONE, 0, LOCATION_REASON_TOFIELD, 0xff) > 0
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_target_group(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let controler = aux::dpcheck(|c, duel| i64::from(c.get_controler(duel)));
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &controler));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &controler, tp, HINTMSG_RTOHAND)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), 2, tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn rthquickconfilter(c: CardId, duel: &Duel, ec: CardId, lg: CardSet) -> bool {
    c.is_link_above(duel, 3)
        && c.is_race(duel, RACE_MACHINE)
        && c.is_faceup(duel)
        && (c.get_linked_group(duel).is_contains(ec) || lg.is_contains(c))
}

fn rthquickcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let lg = c.get_linked_group(duel);
    duel.is_existing_matching_card(
        |t, duel| rthquickconfilter(t, duel, c, lg),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        Some(c.into()),
    )
}
